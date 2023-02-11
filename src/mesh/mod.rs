use crate::{
    gizmo_material::GizmoMaterial, GizmoSettings, InternalGizmoCamera, PickableGizmo,
    TransformGizmoBundle, TransformGizmoInteraction,
};
use bevy::{pbr::NotShadowCaster, prelude::*, render::view::RenderLayers};
use bevy_mod_raycast::NoBackfaceCulling;

mod cone;
mod truncated_torus;

const HIGHLIGHT_AMOUNT: f32 = 0.25;

#[derive(Component)]
pub struct RotationGizmo;

#[derive(Component)]
pub struct ViewTranslateGizmo;

#[derive(Resource)]
pub struct GizmoHighlighter {
    x_arrow_tail: Entity,
    y_arrow_tail: Entity,
    z_arrow_tail: Entity,
    x_arrow_cone: Entity,
    y_arrow_cone: Entity,
    z_arrow_cone: Entity,
    x_plane: Entity,
    y_plane: Entity,
    z_plane: Entity,
    x_rotation_arc: Entity,
    y_rotation_arc: Entity,
    z_rotation_arc: Entity,
    x_mat: Handle<GizmoMaterial>,
    y_mat: Handle<GizmoMaterial>,
    z_mat: Handle<GizmoMaterial>,
    v_mat: Handle<GizmoMaterial>,
    x_mat_highlight: Handle<GizmoMaterial>,
    y_mat_highlight: Handle<GizmoMaterial>,
    z_mat_highlight: Handle<GizmoMaterial>,
    v_mat_highlight: Handle<GizmoMaterial>,
    highlighted: Option<Entity>,
}

impl GizmoHighlighter {
    pub fn get_connected(&self, entity: Entity) -> Vec<Entity> {
        if entity == self.x_arrow_tail || entity == self.x_arrow_cone {
            return vec![self.x_arrow_tail, self.x_arrow_cone];
        }
        if entity == self.y_arrow_tail || entity == self.y_arrow_cone {
            return vec![self.y_arrow_tail, self.y_arrow_cone];
        }
        if entity == self.z_arrow_tail || entity == self.z_arrow_cone {
            return vec![self.z_arrow_tail, self.z_arrow_cone];
        }
        vec![entity]
    }

    fn get_material(&self, entity: Entity, highlighted: bool) -> Handle<GizmoMaterial> {
        if entity == self.x_arrow_tail
            || entity == self.x_arrow_cone
            || entity == self.x_plane
            || entity == self.x_rotation_arc
        {
            return if highlighted {
                &self.x_mat_highlight
            } else {
                &self.x_mat
            }
            .clone();
        }
        if entity == self.y_arrow_tail
            || entity == self.y_arrow_cone
            || entity == self.y_plane
            || entity == self.y_rotation_arc
        {
            return if highlighted {
                &self.y_mat_highlight
            } else {
                &self.y_mat
            }
            .clone();
        }
        if entity == self.z_arrow_tail
            || entity == self.z_arrow_cone
            || entity == self.z_plane
            || entity == self.z_rotation_arc
        {
            return if highlighted {
                &self.z_mat_highlight
            } else {
                &self.z_mat
            }
            .clone();
        }
        return if highlighted {
            &self.v_mat_highlight
        } else {
            &self.v_mat
        }
        .clone();
    }

    pub fn highlight(&mut self, commands: &mut Commands, entity: Entity) {
        if let Some(_) = self.highlighted {
            self.unhighlight(commands);
        }
        for entity in self.get_connected(entity) {
            if let Some(mut ec) = commands.get_entity(entity) {
                ec.insert(self.get_material(entity, true));
            }
        }
        self.highlighted = Some(entity);
    }

    pub fn unhighlight(&mut self, commands: &mut Commands) {
        if let Some(hightlighted) = self.highlighted {
            for entity in self.get_connected(hightlighted) {
                if let Some(mut ec) = commands.get_entity(entity) {
                    ec.insert(self.get_material(entity, false));
                }
            }
            self.highlighted = None;
        }
    }

    pub fn is_highlighted(&self, entity: Entity) -> bool {
        if let Some(highlighted_entity) = self.highlighted {
            return highlighted_entity == entity;
        }
        false
    }
}

/// Startup system that builds the procedural mesh and materials of the gizmo.
pub fn build_gizmo(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<GizmoMaterial>>,
    settings: Res<GizmoSettings>,
) {
    let axis_length = 1.3;
    let arc_radius = 1.;
    let plane_size = axis_length * 0.25;
    let plane_offset = plane_size / 2. + axis_length * 0.2;
    // Define gizmo meshes
    let arrow_tail_mesh = meshes.add(Mesh::from(shape::Capsule {
        radius: 0.05,
        depth: axis_length,
        ..Default::default()
    }));
    let cone_mesh = meshes.add(Mesh::from(cone::Cone {
        height: 0.25,
        radius: 0.12,
        ..Default::default()
    }));
    let plane_mesh = meshes.add(Mesh::from(shape::Plane { size: plane_size }));
    let sphere_mesh = meshes.add(Mesh::from(shape::Icosphere {
        radius: 0.2,
        subdivisions: 3,
    }));
    let rotation_mesh = meshes.add(Mesh::from(truncated_torus::TruncatedTorus {
        radius: arc_radius,
        ring_radius: 0.05,
        ..Default::default()
    }));
    //let cube_mesh = meshes.add(Mesh::from(shape::Cube { size: 0.15 }));
    // Define gizmo materials
    let gizmo_matl_x = materials.add(GizmoMaterial::from(settings.colors.x));
    let gizmo_matl_y = materials.add(GizmoMaterial::from(settings.colors.y));
    let gizmo_matl_z = materials.add(GizmoMaterial::from(settings.colors.z));
    let gizmo_matl_v = materials.add(GizmoMaterial::from(settings.colors.v));
    /*let gizmo_matl_origin = materials.add(StandardMaterial {
        unlit: true,
        base_color: Color::rgb(0.7, 0.7, 0.7),
        ..Default::default()
    });*/
    // Build the gizmo using the variables above.
    let parent = commands.spawn(TransformGizmoBundle::default()).id();

    // Translation Axes
    let x_arrow_tail = commands
        .spawn((
            MaterialMeshBundle {
                mesh: arrow_tail_mesh.clone(),
                material: gizmo_matl_x.clone(),
                transform: Transform::from_matrix(Mat4::from_rotation_translation(
                    Quat::from_rotation_z(std::f32::consts::PI / 2.0),
                    Vec3::new(axis_length / 2.0, 0.0, 0.0),
                )),
                ..Default::default()
            },
            PickableGizmo::default(),
            TransformGizmoInteraction::TranslateAxis {
                original: Vec3::X,
                axis: Vec3::X,
            },
            NotShadowCaster,
            RenderLayers::layer(12),
        ))
        .id();

    let y_arrow_tail = commands
        .spawn((
            MaterialMeshBundle {
                mesh: arrow_tail_mesh.clone(),
                material: gizmo_matl_y.clone(),
                transform: Transform::from_matrix(Mat4::from_rotation_translation(
                    Quat::from_rotation_y(std::f32::consts::PI / 2.0),
                    Vec3::new(0.0, axis_length / 2.0, 0.0),
                )),
                ..Default::default()
            },
            PickableGizmo::default(),
            TransformGizmoInteraction::TranslateAxis {
                original: Vec3::Y,
                axis: Vec3::Y,
            },
            NotShadowCaster,
            RenderLayers::layer(12),
        ))
        .id();

    let z_arrow_tail = commands
        .spawn((
            MaterialMeshBundle {
                mesh: arrow_tail_mesh,
                material: gizmo_matl_z.clone(),
                transform: Transform::from_matrix(Mat4::from_rotation_translation(
                    Quat::from_rotation_x(std::f32::consts::PI / 2.0),
                    Vec3::new(0.0, 0.0, axis_length / 2.0),
                )),
                ..Default::default()
            },
            PickableGizmo::default(),
            TransformGizmoInteraction::TranslateAxis {
                original: Vec3::Z,
                axis: Vec3::Z,
            },
            NotShadowCaster,
            RenderLayers::layer(12),
        ))
        .id();

    let x_arrow_cone = commands
        .spawn((
            MaterialMeshBundle {
                mesh: cone_mesh.clone(),
                material: gizmo_matl_x.clone(),
                transform: Transform::from_matrix(Mat4::from_rotation_translation(
                    Quat::from_rotation_z(std::f32::consts::PI / -2.0),
                    Vec3::new(axis_length, 0.0, 0.0),
                )),
                ..Default::default()
            },
            PickableGizmo::default(),
            TransformGizmoInteraction::TranslateAxis {
                original: Vec3::X,
                axis: Vec3::X,
            },
            NotShadowCaster,
            RenderLayers::layer(12),
        ))
        .id();

    let x_plane = commands
        .spawn((
            MaterialMeshBundle {
                mesh: plane_mesh.clone(),
                material: gizmo_matl_x.clone(),
                transform: Transform::from_matrix(Mat4::from_rotation_translation(
                    Quat::from_rotation_z(std::f32::consts::PI / -2.0),
                    Vec3::new(0., plane_offset, plane_offset),
                )),
                ..Default::default()
            },
            PickableGizmo::default(),
            TransformGizmoInteraction::TranslatePlane {
                original: Vec3::X,
                normal: Vec3::X,
            },
            NoBackfaceCulling,
            NotShadowCaster,
            RenderLayers::layer(12),
        ))
        .id();

    let y_arrow_cone = commands
        .spawn((
            MaterialMeshBundle {
                mesh: cone_mesh.clone(),
                material: gizmo_matl_y.clone(),
                transform: Transform::from_translation(Vec3::new(0.0, axis_length, 0.0)),
                ..Default::default()
            },
            PickableGizmo::default(),
            TransformGizmoInteraction::TranslateAxis {
                original: Vec3::Y,
                axis: Vec3::Y,
            },
            NotShadowCaster,
            RenderLayers::layer(12),
        ))
        .id();

    let y_plane = commands
        .spawn((
            MaterialMeshBundle {
                mesh: plane_mesh.clone(),
                material: gizmo_matl_y.clone(),
                transform: Transform::from_translation(Vec3::new(plane_offset, 0.0, plane_offset)),
                ..Default::default()
            },
            PickableGizmo::default(),
            TransformGizmoInteraction::TranslatePlane {
                original: Vec3::Y,
                normal: Vec3::Y,
            },
            NoBackfaceCulling,
            NotShadowCaster,
            RenderLayers::layer(12),
        ))
        .id();

    let z_arrow_cone = commands
        .spawn((
            MaterialMeshBundle {
                mesh: cone_mesh.clone(),
                material: gizmo_matl_z.clone(),
                transform: Transform::from_matrix(Mat4::from_rotation_translation(
                    Quat::from_rotation_x(std::f32::consts::PI / 2.0),
                    Vec3::new(0.0, 0.0, axis_length),
                )),
                ..Default::default()
            },
            PickableGizmo::default(),
            TransformGizmoInteraction::TranslateAxis {
                original: Vec3::Z,
                axis: Vec3::Z,
            },
            NotShadowCaster,
            RenderLayers::layer(12),
        ))
        .id();

    let z_plane = commands
        .spawn((
            MaterialMeshBundle {
                mesh: plane_mesh.clone(),
                material: gizmo_matl_z.clone(),
                transform: Transform::from_matrix(Mat4::from_rotation_translation(
                    Quat::from_rotation_x(std::f32::consts::PI / 2.0),
                    Vec3::new(plane_offset, plane_offset, 0.0),
                )),
                ..Default::default()
            },
            PickableGizmo::default(),
            TransformGizmoInteraction::TranslatePlane {
                original: Vec3::Z,
                normal: Vec3::Z,
            },
            NoBackfaceCulling,
            NotShadowCaster,
            RenderLayers::layer(12),
        ))
        .id();

    let v_sphere = commands
        .spawn((
            MaterialMeshBundle {
                mesh: sphere_mesh.clone(),
                material: gizmo_matl_v.clone(),
                ..Default::default()
            },
            PickableGizmo::default(),
            TransformGizmoInteraction::TranslatePlane {
                original: Vec3::ZERO,
                normal: Vec3::Z,
            },
            ViewTranslateGizmo,
            NotShadowCaster,
            RenderLayers::layer(12),
        ))
        .id();

    // Rotation Arcs
    let x_rotation_arc = commands
        .spawn((
            MaterialMeshBundle {
                mesh: rotation_mesh.clone(),
                material: gizmo_matl_x.clone(),
                transform: Transform::from_rotation(Quat::from_axis_angle(
                    Vec3::Z,
                    f32::to_radians(90.0),
                )),
                ..Default::default()
            },
            RotationGizmo,
            PickableGizmo::default(),
            TransformGizmoInteraction::RotateAxis {
                original: Vec3::X,
                axis: Vec3::X,
            },
            NotShadowCaster,
            RenderLayers::layer(12),
        ))
        .id();

    let y_rotation_arc = commands
        .spawn((
            MaterialMeshBundle {
                mesh: rotation_mesh.clone(),
                material: gizmo_matl_y.clone(),
                ..Default::default()
            },
            RotationGizmo,
            PickableGizmo::default(),
            TransformGizmoInteraction::RotateAxis {
                original: Vec3::Y,
                axis: Vec3::Y,
            },
            NotShadowCaster,
            RenderLayers::layer(12),
        ))
        .id();

    let z_rotation_arc = commands
        .spawn((
            MaterialMeshBundle {
                mesh: rotation_mesh.clone(),
                material: gizmo_matl_z.clone(),
                transform: Transform::from_rotation(
                    Quat::from_axis_angle(Vec3::Z, f32::to_radians(90.0))
                        * Quat::from_axis_angle(Vec3::X, f32::to_radians(90.0)),
                ),
                ..Default::default()
            },
            RotationGizmo,
            PickableGizmo::default(),
            TransformGizmoInteraction::RotateAxis {
                original: Vec3::Z,
                axis: Vec3::Z,
            },
            NotShadowCaster,
            RenderLayers::layer(12),
        ))
        .id();

    commands.insert_resource(GizmoHighlighter {
        x_arrow_tail,
        y_arrow_tail,
        z_arrow_tail,
        x_arrow_cone,
        y_arrow_cone,
        z_arrow_cone,
        x_plane,
        y_plane,
        z_plane,
        x_rotation_arc,
        y_rotation_arc,
        z_rotation_arc,
        x_mat: gizmo_matl_x,
        y_mat: gizmo_matl_y,
        z_mat: gizmo_matl_z,
        v_mat: gizmo_matl_v,
        x_mat_highlight: materials.add(GizmoMaterial::from(Color::rgb(
            settings.colors.x.r() + HIGHLIGHT_AMOUNT,
            settings.colors.x.g() + HIGHLIGHT_AMOUNT,
            settings.colors.x.b() + HIGHLIGHT_AMOUNT,
        ))),
        y_mat_highlight: materials.add(GizmoMaterial::from(Color::rgb(
            settings.colors.y.r() + HIGHLIGHT_AMOUNT,
            settings.colors.y.g() + HIGHLIGHT_AMOUNT,
            settings.colors.y.b() + HIGHLIGHT_AMOUNT,
        ))),
        z_mat_highlight: materials.add(GizmoMaterial::from(Color::rgb(
            settings.colors.z.r() + HIGHLIGHT_AMOUNT,
            settings.colors.z.g() + HIGHLIGHT_AMOUNT,
            settings.colors.z.b() + HIGHLIGHT_AMOUNT,
        ))),
        v_mat_highlight: materials.add(GizmoMaterial::from(Color::rgb(
            settings.colors.v.r() + HIGHLIGHT_AMOUNT,
            settings.colors.v.g() + HIGHLIGHT_AMOUNT,
            settings.colors.v.b() + HIGHLIGHT_AMOUNT,
        ))),
        highlighted: None,
    });

    let entities = [
        x_arrow_tail,
        y_arrow_tail,
        z_arrow_tail,
        x_arrow_cone,
        y_arrow_cone,
        z_arrow_cone,
        x_plane,
        y_plane,
        z_plane,
        x_rotation_arc,
        y_rotation_arc,
        z_rotation_arc,
        v_sphere,
    ];

    for entity in &entities {
        commands.entity(parent).add_child(*entity);
    }

    commands.spawn((
        Camera3dBundle {
            camera_3d: Camera3d {
                clear_color: bevy::core_pipeline::clear_color::ClearColorConfig::None,
                depth_load_op: bevy::core_pipeline::core_3d::Camera3dDepthLoadOp::Clear(0.),
            },
            ..Default::default()
        },
        InternalGizmoCamera,
        RenderLayers::layer(12),
    ));
}
