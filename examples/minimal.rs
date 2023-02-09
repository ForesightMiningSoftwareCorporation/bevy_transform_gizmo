use bevy::{prelude::*, window::PresentMode::AutoNoVsync};
use bevy_mod_picking::DefaultPickingPlugins;
use bevy_transform_gizmo::{GizmoColors, TransformGizmoPlugin};

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                present_mode: AutoNoVsync,
                ..Default::default()
            },
            ..default()
        }))
        .add_plugins(DefaultPickingPlugins)
        .add_plugin(
            TransformGizmoPlugin::new(
                Quat::from_rotation_y(-0.2), // Align the gizmo to a different coordinate system.
                                             // Use TransformGizmoPlugin::default() to align to the
                                             // scene's coordinate system.
            ),
            //.with_colors(GizmoColors { // Optionally configure colors
            //    x: Color::PINK,
            //    y: Color::YELLOW_GREEN,
            //    z: Color::MIDNIGHT_BLUE,
            //    ..Default::default()
            //}),
        )
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
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            ..Default::default()
        },
        bevy_mod_picking::PickableBundle::default(),
        bevy_transform_gizmo::GizmoTransformable,
    ));
    // cube
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..Default::default()
        },
        bevy_mod_picking::PickableBundle::default(),
        bevy_transform_gizmo::GizmoTransformable,
    ));
    // light
    commands.spawn(PointLightBundle {
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..Default::default()
    });
    // camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        },
        bevy_mod_picking::PickingCameraBundle::default(),
        bevy_transform_gizmo::GizmoPickSource::default(),
    ));
}
