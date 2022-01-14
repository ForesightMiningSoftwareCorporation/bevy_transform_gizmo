use crate::{
    gizmo_material::GizmoMaterial, PickableGizmo, TransformGizmoBundle, TransformGizmoInteraction,
};
use bevy::prelude::*;

mod cone;
mod truncated_torus;

/// Startup system that builds the procedural mesh and materials of the gizmo.
pub fn build_gizmo(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<GizmoMaterial>>,
) {
    let axis_length = 1.5;
    let arc_radius = 1.1;
    // Define gizmo meshes
    let arrow_tail_mesh = meshes.add(Mesh::from(shape::Capsule {
        radius: 0.015,
        depth: axis_length,
        ..Default::default()
    }));
    let cone_mesh = meshes.add(Mesh::from(cone::Cone {
        height: 0.3,
        radius: 0.1,
        ..Default::default()
    }));
    let sphere_mesh = meshes.add(Mesh::from(shape::Icosphere {
        radius: 0.12,
        subdivisions: 8,
    }));
    let rotation_mesh = meshes.add(Mesh::from(truncated_torus::TruncatedTorus {
        radius: arc_radius,
        ring_radius: 0.015,
        ..Default::default()
    }));
    //let cube_mesh = meshes.add(Mesh::from(shape::Cube { size: 0.15 }));
    // Define gizmo materials
    let gizmo_material_x = materials.add(GizmoMaterial::from(Color::rgb(1.0, 0.4, 0.4)));
    let gizmo_material_y = materials.add(GizmoMaterial::from(Color::rgb(0.4, 1.0, 0.4)));
    let gizmo_material_z = materials.add(GizmoMaterial::from(Color::rgb(0.4, 0.5, 1.0)));
    let gizmo_material_x_selectable = materials.add(GizmoMaterial::from(Color::rgb(1.0, 0.7, 0.7)));
    let gizmo_material_y_selectable = materials.add(GizmoMaterial::from(Color::rgb(0.7, 1.0, 0.7)));
    let gizmo_material_z_selectable = materials.add(GizmoMaterial::from(Color::rgb(0.7, 0.7, 1.0)));
    /*let gizmo_material_origin = materials.add(StandardMaterial {
        unlit: true,
        base_color: Color::rgb(0.7, 0.7, 0.7),
        ..Default::default()
    });*/
    // Build the gizmo using the variables above.
    commands
        .spawn_bundle(TransformGizmoBundle::default())
        .with_children(|parent| {
            // Translation Axes
            parent.spawn_bundle(MaterialMeshBundle {
                mesh: arrow_tail_mesh.clone(),
                material: gizmo_material_x.clone(),
                transform: Transform::from_matrix(Mat4::from_rotation_translation(
                    Quat::from_rotation_z(std::f32::consts::PI / 2.0),
                    Vec3::new(axis_length / 2.0, 0.0, 0.0),
                )),
                ..Default::default()
            });
            parent.spawn_bundle(MaterialMeshBundle {
                mesh: arrow_tail_mesh.clone(),
                material: gizmo_material_y.clone(),
                transform: Transform::from_matrix(Mat4::from_rotation_translation(
                    Quat::from_rotation_y(std::f32::consts::PI / 2.0),
                    Vec3::new(0.0, axis_length / 2.0, 0.0),
                )),
                ..Default::default()
            });
            parent.spawn_bundle(MaterialMeshBundle {
                mesh: arrow_tail_mesh,
                material: gizmo_material_z.clone(),
                transform: Transform::from_matrix(Mat4::from_rotation_translation(
                    Quat::from_rotation_x(std::f32::consts::PI / 2.0),
                    Vec3::new(0.0, 0.0, axis_length / 2.0),
                )),
                ..Default::default()
            });

            // Translation Handles
            parent
                .spawn_bundle(MaterialMeshBundle {
                    mesh: cone_mesh.clone(),
                    material: gizmo_material_x_selectable.clone(),
                    transform: Transform::from_matrix(Mat4::from_rotation_translation(
                        Quat::from_rotation_z(std::f32::consts::PI / -2.0),
                        Vec3::new(axis_length, 0.0, 0.0),
                    )),
                    ..Default::default()
                })
                .insert(PickableGizmo::default())
                .insert(TransformGizmoInteraction::TranslateAxis(Vec3::X));
            parent
                .spawn_bundle(MaterialMeshBundle {
                    mesh: cone_mesh.clone(),
                    material: gizmo_material_y_selectable.clone(),
                    transform: Transform::from_translation(Vec3::new(0.0, axis_length, 0.0)),
                    ..Default::default()
                })
                .insert(PickableGizmo::default())
                .insert(TransformGizmoInteraction::TranslateAxis(Vec3::Y));
            parent
                .spawn_bundle(MaterialMeshBundle {
                    mesh: cone_mesh.clone(),
                    material: gizmo_material_z_selectable.clone(),
                    transform: Transform::from_matrix(Mat4::from_rotation_translation(
                        Quat::from_rotation_x(std::f32::consts::PI / 2.0),
                        Vec3::new(0.0, 0.0, axis_length),
                    )),
                    ..Default::default()
                })
                .insert(PickableGizmo::default())
                .insert(TransformGizmoInteraction::TranslateAxis(Vec3::Z));
            /*
                        // Origin
                        parent
                            .spawn_bundle(MaterialMeshBundle {
                                mesh: sphere_mesh.clone(),
                                material: gizmo_material_origin,
                                ..Default::default()
                            })
                            .insert(PickableGizmo::default())
                            .insert(TransformGizmoInteraction::TranslateOrigin);
            */
            // Rotation Arcs
            parent.spawn_bundle(MaterialMeshBundle {
                mesh: rotation_mesh.clone(),
                material: gizmo_material_x.clone(),
                transform: Transform::from_rotation(Quat::from_axis_angle(
                    Vec3::Z,
                    f32::to_radians(90.0),
                )),
                ..Default::default()
            });
            parent.spawn_bundle(MaterialMeshBundle {
                mesh: rotation_mesh.clone(),
                material: gizmo_material_y.clone(),
                ..Default::default()
            });
            parent.spawn_bundle(MaterialMeshBundle {
                mesh: rotation_mesh.clone(),
                material: gizmo_material_z.clone(),
                transform: Transform::from_rotation(
                    Quat::from_axis_angle(Vec3::Z, f32::to_radians(90.0))
                        * Quat::from_axis_angle(Vec3::X, f32::to_radians(90.0)),
                ),
                ..Default::default()
            });

            // Rotation Handles
            parent
                .spawn_bundle(MaterialMeshBundle {
                    mesh: sphere_mesh.clone(),
                    material: gizmo_material_x_selectable.clone(),
                    transform: Transform::from_translation(Vec3::new(
                        0.0,
                        f32::to_radians(45.0).sin() * arc_radius,
                        f32::to_radians(45.0).sin() * arc_radius,
                    )),
                    ..Default::default()
                })
                .insert(PickableGizmo::default())
                .insert(TransformGizmoInteraction::RotateAxis(Vec3::X));
            parent
                .spawn_bundle(MaterialMeshBundle {
                    mesh: sphere_mesh.clone(),
                    material: gizmo_material_y_selectable.clone(),
                    transform: Transform::from_translation(Vec3::new(
                        f32::to_radians(45.0).sin() * arc_radius,
                        0.0,
                        f32::to_radians(45.0).sin() * arc_radius,
                    )),
                    ..Default::default()
                })
                .insert(PickableGizmo::default())
                .insert(TransformGizmoInteraction::RotateAxis(Vec3::Y));
            parent
                .spawn_bundle(MaterialMeshBundle {
                    mesh: sphere_mesh.clone(),
                    material: gizmo_material_z_selectable.clone(),
                    transform: Transform::from_translation(Vec3::new(
                        f32::to_radians(45.0).sin() * arc_radius,
                        f32::to_radians(45.0).sin() * arc_radius,
                        0.0,
                    )),
                    ..Default::default()
                })
                .insert(PickableGizmo::default())
                .insert(TransformGizmoInteraction::RotateAxis(Vec3::Z));
            /*
                        // Scaling Handles
                        parent
                            .spawn_bundle(MaterialMeshBundle {
                                mesh: cube_mesh.clone(),
                                material: gizmo_material_x_selectable.clone(),
                                transform: Transform::from_translation(Vec3::new(arc_radius, 0.0, 0.0)),
                                ..Default::default()
                            })
                            .insert(PickableGizmo::default())
                            .insert(TransformGizmoInteraction::ScaleAxis(Vec3::X));
                        parent
                            .spawn_bundle(MaterialMeshBundle {
                                mesh: cube_mesh.clone(),
                                material: gizmo_material_y_selectable.clone(),
                                transform: Transform::from_translation(Vec3::new(0.0, arc_radius, 0.0)),
                                ..Default::default()
                            })
                            .insert(PickableGizmo::default())
                            .insert(TransformGizmoInteraction::ScaleAxis(Vec3::Y));
                        parent
                            .spawn_bundle(MaterialMeshBundle {
                                mesh: cube_mesh.clone(),
                                material: gizmo_material_z_selectable.clone(),
                                transform: Transform::from_translation(Vec3::new(0.0, 0.0, arc_radius)),
                                ..Default::default()
                            })
                            .insert(PickableGizmo::default())
                            .insert(TransformGizmoInteraction::ScaleAxis(Vec3::Z));
            */
        });
}
