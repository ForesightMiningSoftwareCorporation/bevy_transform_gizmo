use bevy::{prelude::*, window::PresentMode::Mailbox};

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            present_mode: Mailbox,
            ..Default::default()
        })
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugins(bevy_mod_picking::DefaultPickingPlugins)
        .add_plugin(bevy_transform_gizmo::TransformGizmoPlugin::new(
            Quat::from_rotation_y(-0.2), // Align the gizmo to a different coordinate system.
        )) // Use TransformGizmoPlugin::default() to align to the scene's coordinate system.
        .add_startup_system(setup)
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            ..Default::default()
        })
        .insert_bundle(bevy_mod_picking::PickableBundle::default())
        .insert(bevy_transform_gizmo::GizmoTransformable);
    // cube
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..Default::default()
        })
        .insert_bundle(bevy_mod_picking::PickableBundle::default())
        .insert(bevy_transform_gizmo::GizmoTransformable);
    // light
    commands.spawn_bundle(PointLightBundle {
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..Default::default()
    });

    // camera
    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_xyz(2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        })
        .insert_bundle(bevy_mod_picking::PickingCameraBundle::default())
        .insert(bevy_transform_gizmo::GizmoPickSource::default());  //new_transform_empty idk why picking camera doesn't have ray
}
