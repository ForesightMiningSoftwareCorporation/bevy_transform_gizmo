use bevy::ecs::schedule::ShouldRun;
use bevy::{prelude::*, transform::TransformSystem};
use bevy_mod_picking::{
    self, PickingBlocker, PickingCamera, PickingSystem, Primitive3d, Selection,
};
use bevy_mod_raycast::RaycastSystem;
use normalization::*;

pub use picking::{GizmoPickSource, PickableGizmo};

mod mesh;
mod normalization;
pub mod picking;
mod render_graph;

pub struct GizmoSystemsEnabled(pub bool);
pub use normalization::Ui3dNormalization;

#[derive(Clone, Hash, PartialEq, Eq, Debug, RunCriteriaLabel)]
pub struct GizmoSystemsEnabledCriteria;

fn plugin_enabled(enabled: Res<GizmoSystemsEnabled>) -> ShouldRun {
    if enabled.0 {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemLabel)]
pub enum TransformGizmoSystem {
    UpdateSettings,
    Place,
    Hover,
    Grab,
    Drag,
}

#[derive(Debug)]
pub struct TransformGizmoEvent {
    pub from: GlobalTransform,
    pub to: GlobalTransform,
    pub interaction: TransformGizmoInteraction,
}

#[derive(Component)]
pub struct GizmoTransformable;

pub struct GizmoSettings {
    /// Rotation to apply to the gizmo when it is placed. Used to align the gizmo to a different
    /// coordinate system.
    alignment_rotation: Quat,
}

#[derive(Default)]
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
        let alignment_rotation = self.alignment_rotation;
        app.insert_resource(GizmoSettings { alignment_rotation })
            .add_event::<TransformGizmoEvent>()
            .add_startup_system(mesh::build_gizmo)
            .add_startup_system_to_stage(StartupStage::PostStartup, place_gizmo)
            .insert_resource(GizmoSystemsEnabled(true))
            .add_plugin(picking::GizmoPickingPlugin)
            .add_system_set_to_stage(
                CoreStage::PreUpdate,
                SystemSet::new()
                    .with_run_criteria(plugin_enabled.label(GizmoSystemsEnabledCriteria))
                    .with_system(update_gizmo_alignment.label(TransformGizmoSystem::UpdateSettings))
                    .with_system(
                        hover_gizmo
                            .label(TransformGizmoSystem::Hover)
                            .after(TransformGizmoSystem::UpdateSettings)
                            .after(RaycastSystem::UpdateRaycast),
                    )
                    .with_system(
                        grab_gizmo
                            .label(TransformGizmoSystem::Grab)
                            .after(TransformGizmoSystem::Hover)
                            .before(PickingSystem::PauseForBlockers),
                    ),
            )
            .add_system_set_to_stage(
                CoreStage::PostUpdate,
                SystemSet::new()
                    .with_run_criteria(plugin_enabled.label(GizmoSystemsEnabledCriteria))
                    .with_system(
                        drag_gizmo
                            .label(TransformGizmoSystem::Drag)
                            .before(TransformSystem::TransformPropagate),
                    )
                    .with_system(
                        place_gizmo
                            .label(TransformGizmoSystem::Place)
                            .after(TransformSystem::TransformPropagate)
                            .after(TransformGizmoSystem::Drag),
                    ),
            );
        {
            render_graph::add_gizmo_graph(&mut app.world);
        }
    }
}

#[derive(Bundle)]
pub struct TransformGizmoBundle {
    gizmo: TransformGizmo,
    interaction: Interaction,
    picking_blocker: PickingBlocker,
    transform: Transform,
    global_transform: GlobalTransform,
    visible: Visible,
    normalize: Normalize3d,
}

impl Default for TransformGizmoBundle {
    fn default() -> Self {
        TransformGizmoBundle {
            transform: Transform::from_translation(Vec3::splat(f32::MIN)),
            interaction: Interaction::None,
            picking_blocker: PickingBlocker,
            visible: Visible {
                is_visible: false,
                is_transparent: false,
            },
            gizmo: TransformGizmo::default(),
            global_transform: GlobalTransform::default(),
            normalize: Normalize3d::new(1.5, 150.0),
        }
    }
}

#[derive(Component, Default, PartialEq)]
pub struct TransformGizmo {
    current_interaction: Option<TransformGizmoInteraction>,
    // Point in space where mouse-gizmo interaction started (on mouse down), used to compare how
    // much total dragging has occurred without accumulating error across frames.
    drag_start: Option<Vec3>,
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
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub enum TransformGizmoInteraction {
    TranslateAxis { original: Vec3, axis: Vec3 },
    TranslateOrigin,
    RotateAxis { original: Vec3, axis: Vec3 },
    ScaleAxis { original: Vec3, axis: Vec3 },
}

#[derive(Component)]
struct InitialTransform {
    transform: GlobalTransform,
}

/// Updates the position of the gizmo and selected meshes while the gizmo is being dragged.
#[allow(clippy::type_complexity)]
fn drag_gizmo(
    pick_cam: Query<&PickingCamera>,
    mut gizmo_mut: Query<&mut TransformGizmo>,
    mut transform_queries: QuerySet<(
        QueryState<(&Selection, &mut Transform, &InitialTransform)>,
        QueryState<&Transform, With<TransformGizmo>>,
        QueryState<(&GlobalTransform, &Interaction), With<TransformGizmo>>,
    )>,
) {
    // Gizmo handle should project mouse motion onto the axis of the handle. Perpendicular motion
    // should have no effect on the handle. We can do this by projecting the vector from the handle
    // click point to mouse's current position, onto the axis of the direction we are dragging. See
    // the wiki article for details: https://en.wikipedia.org/wiki/Vector_projection
    let gizmo_global_transform = if let Ok((global_transform, &Interaction::Clicked)) =
        transform_queries.q2().get_single()
    {
        global_transform.to_owned()
    } else {
        return;
    };
    let mut gizmo = if let Ok(g) = gizmo_mut.get_single_mut() {
        g
    } else {
        error!("Number of transform gizmos is != 1");
        return;
    };
    let gizmo_origin = gizmo_global_transform.translation;
    let picking_camera = if let Some(cam) = pick_cam.iter().last() {
        cam
    } else {
        return;
    };
    let picking_ray = if let Some(ray) = picking_camera.ray() {
        ray
    } else {
        return;
    };
    if let Some(interaction) = gizmo.current_interaction {
        let gizmo_transform = *transform_queries
            .q1()
            .get_single_mut()
            .expect("Gizmo missing a `Transform` when there is some gizmo interaction.");
        match &gizmo.initial_transform {
            Some(_) => (),
            None => {
                gizmo.initial_transform = Some(gizmo_transform.into());
            }
        };
        match interaction {
            TransformGizmoInteraction::TranslateAxis { original: _, axis } => {
                let cursor_plane_intersection = if let Some(intersection) = picking_camera
                    .intersect_primitive(Primitive3d::Plane {
                        normal: picking_ray.direction(),
                        point: gizmo_origin,
                    }) {
                    intersection.position()
                } else {
                    return;
                };
                let cursor_vector: Vec3 = cursor_plane_intersection - gizmo_origin;
                let drag_start = match &gizmo.drag_start {
                    Some(drag_start) => *drag_start,
                    None => {
                        let handle_vector = axis;
                        let cursor_projected_onto_handle = cursor_vector
                            .dot(handle_vector.normalize())
                            * handle_vector.normalize();
                        gizmo.drag_start = Some(gizmo_origin + cursor_projected_onto_handle);
                        return;
                    }
                };
                let selected_handle_vec = drag_start - gizmo_origin;
                let new_handle_vec = cursor_vector.dot(selected_handle_vec.normalize())
                    * selected_handle_vec.normalize();
                let translation = new_handle_vec - selected_handle_vec;
                transform_queries
                    .q0()
                    .iter_mut()
                    .filter(|(s, _t, _i)| s.selected())
                    .for_each(|(_s, mut t, i)| {
                        *t = Transform {
                            translation: i.transform.translation + translation,
                            rotation: i.transform.rotation,
                            scale: i.transform.scale,
                        }
                    });
            }
            TransformGizmoInteraction::TranslateOrigin => (),
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
                transform_queries
                    .q0()
                    .iter_mut()
                    .filter(|(s, _t, _i)| s.selected())
                    .for_each(|(_s, mut t, i)| {
                        *t = Transform {
                            translation: i.transform.translation,
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
    gizmo_raycast_source: Query<&picking::GizmoPickSource>,
    mut gizmo_query: Query<(&Children, &mut TransformGizmo, &mut Interaction, &Transform)>,
    hover_query: Query<&TransformGizmoInteraction>,
) {
    for (children, mut gizmo, mut interaction, _transform) in gizmo_query.iter_mut() {
        if let Some((topmost_gizmo_entity, _)) = gizmo_raycast_source
            .get_single()
            .expect("Missing gizmo raycast source")
            .intersect_top()
        {
            if *interaction == Interaction::None {
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

/// Tracks when one of the gizmo handles has been clicked on.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn grab_gizmo(
    mut commands: Commands,
    mouse_button_input: Res<Input<MouseButton>>,
    mut gizmo_events: EventWriter<TransformGizmoEvent>,
    mut gizmo_query: Query<(&mut TransformGizmo, &mut Interaction, &GlobalTransform)>,
    selected_items_query: Query<(&Selection, &GlobalTransform, Entity)>,
    initial_transform_query: Query<Entity, With<InitialTransform>>,
) {
    if mouse_button_input.just_pressed(MouseButton::Left) {
        for (mut gizmo, mut interaction, _transform) in gizmo_query.iter_mut() {
            if *interaction == Interaction::Hovered {
                *interaction = Interaction::Clicked;
                // Dragging has started, store the initial position of all selected meshes
                for (selection, transform, entity) in selected_items_query.iter() {
                    if selection.selected() {
                        commands.entity(entity).insert(InitialTransform {
                            transform: *transform,
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
    mut queries: QuerySet<(
        QueryState<(&Selection, &GlobalTransform), With<GizmoTransformable>>,
        QueryState<(&mut GlobalTransform, &mut Transform, &mut Visible), With<TransformGizmo>>,
    )>,
) {
    // Maximum xyz position of all selected entities
    let position = queries
        .q0()
        .iter()
        .filter(|(s, _t)| s.selected())
        .map(|(_s, t)| t)
        .fold(Vec3::splat(f32::MIN), |a, b| a.max(b.translation));
    // Number of selected items
    let selected_items = queries.q0().iter().filter(|(s, _t)| s.selected()).count();
    // Set the gizmo's position and visibility
    if let Ok((mut g_transform, mut transform, mut visible)) = queries.q1().get_single_mut() {
        g_transform.translation = position;
        g_transform.rotation = plugin_settings.alignment_rotation;
        transform.translation = g_transform.translation;
        transform.rotation = g_transform.rotation;
        visible.is_visible = selected_items > 0;
    } else {
        error!("Number of gizmos is != 1");
    }
}

/// Updates the gizmo axes rotation based on the gizmo settings
fn update_gizmo_alignment(
    plugin_settings: Res<GizmoSettings>,
    mut query: Query<&mut TransformGizmoInteraction>,
) {
    let rotation = plugin_settings.alignment_rotation;
    for mut interaction in query.iter_mut() {
        if let Some(rotated_interaction) = match *interaction {
            TransformGizmoInteraction::TranslateAxis { original, axis: _ } => {
                Some(TransformGizmoInteraction::TranslateAxis {
                    original,
                    axis: rotation.mul_vec3(original),
                })
            }
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
            _ => None,
        } {
            *interaction = rotated_interaction;
        }
    }
}
