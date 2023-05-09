#![allow(clippy::type_complexity)]

use bevy::{prelude::*, render::camera::Projection, transform::TransformSystem};
use bevy_mod_picking::selection::PickSelection;
use bevy_mod_raycast::{Primitive3d, RaycastSource, RaycastSystem};
use gizmo_material::GizmoMaterial;
use mesh::{RotationGizmo, ViewTranslateGizmo};
use normalization::*;

mod gizmo_material;
mod mesh;
pub mod normalization;

pub mod picking;
use picking::GizmoRaycastSet;
pub use picking::{GizmoPickSource, PickableGizmo};

#[derive(Resource, Clone, Debug)]
pub struct GizmoSystemsEnabled(pub bool);
pub use normalization::Ui3dNormalization;

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum TransformGizmoSystem {
    InputsSet,
    MainSet,
    RaycastSet,
    NormalizeSet,
    UpdateSettings,
    AdjustViewTranslateGizmo,
    Place,
    Hover,
    Grab,
    Drag,
}

#[derive(Debug, Clone)]
pub struct TransformGizmoEvent {
    pub from: GlobalTransform,
    pub to: GlobalTransform,
    pub interaction: TransformGizmoInteraction,
}

#[derive(Component, Default, Clone, Debug)]
pub struct GizmoTransformable;

#[derive(Component, Default, Clone, Debug)]
pub struct InternalGizmoCamera;

#[derive(Component, Default, Clone, Debug)]
pub struct PickingBlocker;

#[derive(Resource, Clone, Debug)]
pub struct GizmoSettings {
    pub enabled: bool,
    /// Rotation to apply to the gizmo when it is placed. Used to align the gizmo to a different
    /// coordinate system.
    pub alignment_rotation: Quat,
    pub allow_rotation: bool,
}

#[derive(Default, Debug, Clone)]
pub struct TransformGizmoPlugin {
    // Rotation to apply to the gizmo when it is placed. Used to align the gizmo to a different
    // coordinate system.
    alignment_rotation: Quat,
}
impl TransformGizmoPlugin {
    pub fn new(alignment_rotation: Quat) -> Self {
        TransformGizmoPlugin { alignment_rotation }
    }
}
impl Plugin for TransformGizmoPlugin {
    fn build(&self, app: &mut App) {
        let mut shaders = app.world.get_resource_mut::<Assets<Shader>>().unwrap();
        shaders.set_untracked(
            gizmo_material::GIZMO_SHADER_HANDLE,
            Shader::from_wgsl(include_str!("../assets/gizmo_material.wgsl")),
        );
        let alignment_rotation = self.alignment_rotation;
        app.insert_resource(GizmoSettings {
            enabled: true,
            alignment_rotation,
            allow_rotation: true,
        })
        .insert_resource(GizmoSystemsEnabled(true))
        .add_plugin(MaterialPlugin::<GizmoMaterial>::default())
        .add_plugin(picking::GizmoPickingPlugin)
        .add_event::<TransformGizmoEvent>()
        .add_plugin(Ui3dNormalization);

        // Input Set
        app.add_systems(
            (
                update_gizmo_settings.in_set(TransformGizmoSystem::UpdateSettings),
                hover_gizmo
                    .in_set(TransformGizmoSystem::Hover)
                    .after(RaycastSystem::UpdateRaycast::<GizmoRaycastSet>),
                grab_gizmo.in_set(TransformGizmoSystem::Grab),
            )
                .chain()
                .in_set(TransformGizmoSystem::InputsSet),
        )
        .configure_set(
            TransformGizmoSystem::InputsSet
                .run_if(|settings: Res<GizmoSettings>| settings.enabled)
                .in_base_set(CoreSet::PreUpdate),
        );

        // Main Set
        app.add_systems(
            (
                drag_gizmo
                    .in_set(TransformGizmoSystem::Drag)
                    .before(TransformSystem::TransformPropagate),
                place_gizmo
                    .in_set(TransformGizmoSystem::Place)
                    .after(TransformSystem::TransformPropagate),
                propagate_gizmo_elements,
                adjust_view_translate_gizmo.in_set(TransformGizmoSystem::Drag),
                gizmo_cam_copy_settings.in_set(TransformGizmoSystem::Drag),
            )
                .chain()
                .in_set(TransformGizmoSystem::MainSet),
        )
        .configure_set(
            TransformGizmoSystem::MainSet
                .run_if(|settings: Res<GizmoSettings>| settings.enabled)
                .in_base_set(CoreSet::PostUpdate),
        );

        app.add_startup_system(mesh::build_gizmo)
            .add_system(place_gizmo.in_base_set(StartupSet::PostStartup));

        app.add_systems((
            process_new_transformable.before(apply_picking_blocker),            
            apply_picking_blocker.after(process_new_transformable),   
            process_new_camera, 
        ));

    }
}

#[derive(Bundle)]
pub struct TransformGizmoBundle {
    gizmo: TransformGizmo,
    interaction: Interaction,
    picking_blocker: PickingBlocker,
    transform: Transform,
    global_transform: GlobalTransform,
    visible: Visibility,
    computed_visibility: ComputedVisibility,
    normalize: Normalize3d,
}

impl Default for TransformGizmoBundle {
    fn default() -> Self {
        TransformGizmoBundle {
            transform: Transform::from_translation(Vec3::splat(f32::MIN)),
            interaction: Interaction::None,
            picking_blocker: PickingBlocker,
            visible: Visibility::Hidden,
            computed_visibility: ComputedVisibility::default(),
            gizmo: TransformGizmo::default(),
            global_transform: GlobalTransform::default(),
            normalize: Normalize3d::new(1.5, 150.0),
        }
    }
}

#[derive(Default, PartialEq, Component)]
pub struct TransformGizmo {
    current_interaction: Option<TransformGizmoInteraction>,
    // Point in space where mouse-gizmo interaction started (on mouse down), used to compare how
    // much total dragging has occurred without accumulating error across frames.
    drag_start: Option<Vec3>,
    origin_drag_start: Option<Vec3>,
    // Initial transform of the gizmo
    initial_transform: Option<GlobalTransform>,
}

impl TransformGizmo {
    /// Get the gizmo's drag direction.
    pub fn current_interaction(&self) -> Option<TransformGizmoInteraction> {
        self.current_interaction
    }
}

/// Marks the current active gizmo interaction
#[derive(Clone, Copy, Debug, PartialEq, Component)]
pub enum TransformGizmoInteraction {
    TranslateAxis { original: Vec3, axis: Vec3 },
    TranslatePlane { original: Vec3, normal: Vec3 },
    RotateAxis { original: Vec3, axis: Vec3 },
    ScaleAxis { original: Vec3, axis: Vec3 },
}

#[derive(Component)]
struct InitialTransform {
    transform: Transform,
    rotation_offset: Vec3,
}

/// Updates the position of the gizmo and selected meshes while the gizmo is being dragged.
#[allow(clippy::type_complexity)]
fn drag_gizmo(
    pick_cam: Query<&RaycastSource<GizmoRaycastSet>>,
    mut gizmo_mut: Query<&mut TransformGizmo>,
    mut transform_query: Query<
        (&PickSelection, &mut Transform, &InitialTransform),
        Without<TransformGizmo>,
    >,
    gizmo_query: Query<(&GlobalTransform, &Interaction), With<TransformGizmo>>,
) {
    let picking_camera = if let Some(cam) = pick_cam.iter().last() {
        cam
    } else {
        error!("Not exactly one picking camera.");
        return;
    };

    let picking_ray = if let Some(ray) = picking_camera.get_ray() {
        ray
    } else {
        error!("Picking camera does not have a ray.");
        return;
    };
    // Gizmo handle should project mouse motion onto the axis of the handle. Perpendicular motion
    // should have no effect on the handle. We can do this by projecting the vector from the handle
    // click point to mouse's current position, onto the axis of the direction we are dragging. See
    // the wiki article for details: https://en.wikipedia.org/wiki/Vector_projection
    let gizmo_transform = if let Ok((transform, &Interaction::Clicked)) = gizmo_query.get_single() {
        transform.to_owned()
    } else {
        return;
    };
    let mut gizmo = if let Ok(g) = gizmo_mut.get_single_mut() {
        g
    } else {
        error!("Number of transform gizmos is != 1");
        return;
    };
    let gizmo_origin = match gizmo.origin_drag_start {
        Some(origin) => origin,
        None => {
            let origin = gizmo_transform.translation();
            gizmo.origin_drag_start = Some(origin);
            origin
        }
    };
    let selected_iter = transform_query.iter_mut().filter(|(s, ..)| s.is_selected);
    if let Some(interaction) = gizmo.current_interaction {
        if gizmo.initial_transform.is_none() {
            gizmo.initial_transform = Some(gizmo_transform);
        }
        match interaction {
            TransformGizmoInteraction::TranslateAxis { original: _, axis } => {
                let vertical_vector = picking_ray.direction().cross(axis).normalize();
                let plane_normal = axis.cross(vertical_vector).normalize();
                let plane_origin = gizmo_origin;
                let cursor_plane_intersection = if let Some(intersection) = picking_camera
                    .intersect_primitive(Primitive3d::Plane {
                        normal: plane_normal,
                        point: plane_origin,
                    }) {
                    intersection.position()
                } else {
                    return;
                };
                let cursor_vector: Vec3 = cursor_plane_intersection - plane_origin;
                let cursor_projected_onto_handle = match &gizmo.drag_start {
                    Some(drag_start) => *drag_start,
                    None => {
                        let handle_vector = axis;
                        let cursor_projected_onto_handle = cursor_vector
                            .dot(handle_vector.normalize())
                            * handle_vector.normalize();
                        gizmo.drag_start = Some(cursor_projected_onto_handle + plane_origin);
                        return;
                    }
                };
                let selected_handle_vec = cursor_projected_onto_handle - plane_origin;
                let new_handle_vec = cursor_vector.dot(selected_handle_vec.normalize())
                    * selected_handle_vec.normalize();
                let translation = new_handle_vec - selected_handle_vec;
                selected_iter.for_each(|(_s, mut t, i)| {
                    *t = Transform {
                        translation: i.transform.translation + translation,
                        rotation: i.transform.rotation,
                        scale: i.transform.scale,
                    }
                });
            }
            TransformGizmoInteraction::TranslatePlane { normal, .. } => {
                let plane_origin = gizmo_origin;
                let cursor_plane_intersection = if let Some(intersection) = picking_camera
                    .intersect_primitive(Primitive3d::Plane {
                        normal,
                        point: plane_origin,
                    }) {
                    intersection.position()
                } else {
                    return;
                };
                let drag_start = match gizmo.drag_start {
                    Some(drag_start) => drag_start,
                    None => {
                        gizmo.drag_start = Some(cursor_plane_intersection);
                        return;
                    }
                };
                selected_iter.for_each(|(_s, mut t, i)| {
                    *t = Transform {
                        translation: i.transform.translation + cursor_plane_intersection
                            - drag_start,
                        rotation: i.transform.rotation,
                        scale: i.transform.scale,
                    }
                });
            }
            TransformGizmoInteraction::RotateAxis { original: _, axis } => {
                let rotation_plane = Primitive3d::Plane {
                    normal: axis.normalize(),
                    point: gizmo_origin,
                };
                let cursor_plane_intersection = if let Some(intersection) =
                    picking_camera.intersect_primitive(rotation_plane)
                {
                    intersection.position()
                } else {
                    return;
                };
                let cursor_vector = (cursor_plane_intersection - gizmo_origin).normalize();
                let drag_start = match &gizmo.drag_start {
                    Some(drag_start) => *drag_start,
                    None => {
                        gizmo.drag_start = Some(cursor_vector);
                        return; // We just started dragging, no transformation is needed yet, exit early.
                    }
                };
                let dot = drag_start.dot(cursor_vector);
                let det = axis.dot(drag_start.cross(cursor_vector));
                let angle = det.atan2(dot);
                let rotation = Quat::from_axis_angle(axis, angle);
                selected_iter.for_each(|(_s, mut t, i)| {
                    let worldspace_offset = i.transform.rotation * i.rotation_offset;
                    let offset_rotated = rotation * worldspace_offset;
                    let offset = worldspace_offset - offset_rotated;
                    *t = Transform {
                        translation: i.transform.translation + offset,
                        rotation: rotation * i.transform.rotation,
                        scale: i.transform.scale,
                    }
                });
            }
            TransformGizmoInteraction::ScaleAxis {
                original: _,
                axis: _,
            } => (),
        }
    }
}

fn hover_gizmo(
    gizmo_raycast_source: Query<&GizmoPickSource>,
    mut gizmo_query: Query<(&Children, &mut TransformGizmo, &mut Interaction, &Transform)>,
    hover_query: Query<&TransformGizmoInteraction>,
) {
    for (children, mut gizmo, mut interaction, _transform) in gizmo_query.iter_mut() {
        if let Some((topmost_gizmo_entity, _)) = gizmo_raycast_source
            .get_single()
            .expect("Missing gizmo raycast source")
            .get_nearest_intersection()
        {
            // Only update the gizmo state if it isn't being clicked (dragged) currently.
            if *interaction != Interaction::Clicked {
                for child in children
                    .iter()
                    .filter(|entity| **entity == topmost_gizmo_entity)
                {
                    *interaction = Interaction::Hovered;
                    if let Ok(gizmo_interaction) = hover_query.get(*child) {
                        gizmo.current_interaction = Some(*gizmo_interaction);
                    }
                }
            }
        } else if *interaction == Interaction::Hovered {
            *interaction = Interaction::None
        }
    }
}

#[derive(Component)]
pub struct RotationOriginOffset(pub Vec3);

/// Tracks when one of the gizmo handles has been clicked on.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn grab_gizmo(
    mut commands: Commands,
    mouse_button_input: Res<Input<MouseButton>>,
    mut gizmo_events: EventWriter<TransformGizmoEvent>,
    mut gizmo_query: Query<(&mut TransformGizmo, &mut Interaction, &GlobalTransform)>,
    selected_items_query: Query<(
        &PickSelection,
        &Transform,
        Entity,
        Option<&RotationOriginOffset>,
    )>,
    initial_transform_query: Query<Entity, With<InitialTransform>>,
) {
    if mouse_button_input.just_pressed(MouseButton::Left) {
        for (mut gizmo, mut interaction, _transform) in gizmo_query.iter_mut() {
            if *interaction == Interaction::Hovered {
                *interaction = Interaction::Clicked;
                // Dragging has started, store the initial position of all selected meshes
                for (selection, transform, entity, rotation_origin_offset) in
                    selected_items_query.iter()
                {
                    if selection.is_selected {
                        commands.entity(entity).insert(InitialTransform {
                            transform: transform.clone(),
                            rotation_offset: rotation_origin_offset
                                .map(|offset| offset.0)
                                .unwrap_or(Vec3::ZERO),
                        });
                    }
                }
            } else {
                *gizmo = TransformGizmo::default();
                for entity in initial_transform_query.iter() {
                    commands.entity(entity).remove::<InitialTransform>();
                }
            }
        }
    } else if mouse_button_input.just_released(MouseButton::Left) {
        for (mut gizmo, mut interaction, transform) in gizmo_query.iter_mut() {
            *interaction = Interaction::None;
            if let (Some(from), Some(interaction)) =
                (gizmo.initial_transform, gizmo.current_interaction())
            {
                let event = TransformGizmoEvent {
                    from,
                    to: *transform,
                    interaction,
                };
                //info!("{:?}", event);
                gizmo_events.send(event);
                *gizmo = TransformGizmo::default();
            }
        }
    }
}

/// Places the gizmo in space relative to the selected entity(s).
#[allow(clippy::type_complexity)]
fn place_gizmo(
    plugin_settings: Res<GizmoSettings>,
    mut queries: ParamSet<(
        Query<
            (
                &PickSelection,
                &GlobalTransform,
                Option<&RotationOriginOffset>,
            ),
            With<GizmoTransformable>,
        >,
        Query<(&mut GlobalTransform, &mut Transform, &mut Visibility), With<TransformGizmo>>,
    )>,
) {
    let selected: Vec<_> = queries
        .p0()
        .iter()
        .filter(|(s, ..)| s.is_selected)
        .map(|(_s, t, offset)| {
            t.translation()
                + offset
                    .map(|o| t.compute_transform().rotation * o.0)
                    .unwrap_or(Vec3::ZERO)
        })
        .collect();
    let n_selected = selected.len();
    let transform_sum = selected.iter().fold(Vec3::ZERO, |acc, t| acc + *t);
    let centroid = transform_sum / n_selected as f32;
    // Set the gizmo's position and visibility
    if let Ok((mut g_transform, mut transform, mut visible)) = queries.p1().get_single_mut() {
        let gt = g_transform.compute_transform();
        *g_transform = Transform {
            translation: centroid,
            rotation: plugin_settings.alignment_rotation,
            ..gt
        }
        .into();
        transform.translation = centroid;
        transform.rotation = plugin_settings.alignment_rotation;
        if n_selected > 0 {
            *visible = Visibility::Inherited;
        } else {
            *visible = Visibility::Hidden;
        }
    } else {
        error!("Number of gizmos is != 1");
    }
}

fn propagate_gizmo_elements(
    gizmo: Query<(&GlobalTransform, &Children), With<TransformGizmo>>,
    mut gizmo_parts_query: Query<(&Transform, &mut GlobalTransform), Without<TransformGizmo>>,
) {
    if let Ok((gizmo_pos, gizmo_parts)) = gizmo.get_single() {
        for &entity in gizmo_parts.iter() {
            let (transform, mut g_transform) = gizmo_parts_query.get_mut(entity).unwrap();
            *g_transform = gizmo_pos.mul_transform(*transform);
        }
    }
}

fn update_gizmo_settings(
    plugin_settings: Res<GizmoSettings>,
    mut interactions: Query<&mut TransformGizmoInteraction, Without<ViewTranslateGizmo>>,
    mut rotations: Query<&mut Visibility, With<RotationGizmo>>,
) {
    if !plugin_settings.is_changed() {
        return;
    }
    let rotation = plugin_settings.alignment_rotation;
    for mut interaction in interactions.iter_mut() {
        if let Some(rotated_interaction) = match *interaction {
            TransformGizmoInteraction::TranslateAxis { original, axis: _ } => {
                Some(TransformGizmoInteraction::TranslateAxis {
                    original,
                    axis: rotation.mul_vec3(original),
                })
            }
            TransformGizmoInteraction::TranslatePlane {
                original,
                normal: _,
            } => Some(TransformGizmoInteraction::TranslatePlane {
                original,
                normal: rotation.mul_vec3(original),
            }),
            TransformGizmoInteraction::RotateAxis { original, axis: _ } => {
                Some(TransformGizmoInteraction::RotateAxis {
                    original,
                    axis: rotation.mul_vec3(original),
                })
            }
            TransformGizmoInteraction::ScaleAxis { original, axis: _ } => {
                Some(TransformGizmoInteraction::ScaleAxis {
                    original,
                    axis: rotation.mul_vec3(original),
                })
            }
        } {
            *interaction = rotated_interaction;
        }
    }

    for mut visibility in rotations.iter_mut() {
        if plugin_settings.allow_rotation {
            *visibility = Visibility::Inherited;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}

#[allow(clippy::type_complexity)]
fn adjust_view_translate_gizmo(
    mut gizmo: Query<
        (&mut GlobalTransform, &mut TransformGizmoInteraction),
        (With<ViewTranslateGizmo>, Without<GizmoPickSource>),
    >,
    camera: Query<&Transform, With<GizmoPickSource>>,
) {
    let (mut global_transform, mut interaction) = match gizmo.get_single_mut() {
        Ok(x) => x,
        Err(_) => return,
    };

    let cam_transform = match camera.get_single() {
        Ok(x) => x,
        Err(_) => return,
    };

    let direction = cam_transform.local_z();
    *interaction = TransformGizmoInteraction::TranslatePlane {
        original: Vec3::ZERO,
        normal: direction,
    };
    let rotation = Quat::from_mat3(&Mat3::from_cols(
        direction.cross(cam_transform.local_y()),
        direction,
        cam_transform.local_y(),
    ));
    *global_transform = Transform {
        rotation,
        ..global_transform.compute_transform()
    }
    .into();
}

fn gizmo_cam_copy_settings(
    main_cam: Query<(Ref<Camera>, Ref<GlobalTransform>, Ref<Projection>), With<GizmoPickSource>>,
    mut gizmo_cam: Query<
        (&mut Camera, &mut GlobalTransform, &mut Projection),
        (With<InternalGizmoCamera>, Without<GizmoPickSource>),
    >,
) {
    let (main_cam, main_cam_pos, main_proj) = if let Ok(x) = main_cam.get_single() {
        x
    } else {
        error!("No `GizmoPickSource` found! Insert the `GizmoPickSource` component onto your primary 3d camera");
        return;
    };
    let (mut gizmo_cam, mut gizmo_cam_pos, mut proj) = gizmo_cam.single_mut();
    if main_cam_pos.is_changed() {
        *gizmo_cam_pos = *main_cam_pos;
    }
    if main_cam.is_changed() {
        *gizmo_cam = main_cam.clone();
        gizmo_cam.order += 10;
    }
    if main_proj.is_changed() {
        *proj = main_proj.clone();
    }
}

fn apply_picking_blocker (
    mut commands: Commands,
    entities: Query<Entity, With<PickingBlocker>>,    
) {
    for entity in entities.iter() {
        if let Some(mut entity_commands) = commands.get_entity(entity) {
            entity_commands.remove::<PickingBlocker>();
            entity_commands.remove::<bevy_mod_picking::selection::PickSelection>();
        } 
    } 
}

fn process_new_transformable (
    mut commands: Commands,
    new_transformable_entities: Query<Entity, Added<GizmoTransformable>>,    
    entities_with_blocker: Query<&PickingBlocker>,  
) {
    for entity in new_transformable_entities.iter() {
        if let Some(mut entity_commands) = commands.get_entity(entity) {
            entity_commands.insert(bevy_mod_picking::PickableBundle::default());
            entity_commands.insert(bevy_mod_picking::prelude::RaycastPickTarget::default());

            if entities_with_blocker.contains(entity) {
                entity_commands.remove::<PickingBlocker>();
                entity_commands.remove::<bevy_mod_picking::selection::PickSelection>();
            }
        } 
    }
}

fn process_new_camera (
    mut commands: Commands,
//    new_transformable_camera: Query<Entity, Added<GizmoPickSource>>,    
    new_transformable_camera: Query<Entity, (With<GizmoPickSource>, With<Camera>, Without<bevy_mod_picking::prelude::RaycastPickCamera>)>,   
) {
    for entity in new_transformable_camera.iter() {
        if let Some(mut entity_commands) = commands.get_entity(entity) {
            entity_commands.insert(bevy_mod_picking::prelude::RaycastPickCamera::default());
        } 
    }
}


