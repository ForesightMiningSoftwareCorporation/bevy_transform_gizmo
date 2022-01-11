use bevy::{prelude::*, render::camera::Camera, transform::TransformSystem};

use crate::{GizmoSystemsEnabledCriteria, TransformGizmoSystem};

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemLabel)]
pub enum FseNormalizeSystem {
    Normalize,
}

pub struct Ui3dNormalization;
impl Plugin for Ui3dNormalization {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(
            CoreStage::PostUpdate,
            normalize
                .label(FseNormalizeSystem::Normalize)
                .with_run_criteria(GizmoSystemsEnabledCriteria)
                .after(TransformSystem::TransformPropagate)
                .after(TransformGizmoSystem::Place),
        );
    }
}

/// Marker struct that marks entities with meshes that should be scaled relative to the camera.
#[derive(Component)]
pub struct Normalize3d {
    /// Length of the object in world space units
    pub size_in_world: f32,
    /// Desired length of the object in pixels
    pub desired_pixel_size: f32,
}
impl Normalize3d {
    pub fn new(size_in_world: f32, desired_pixel_size: f32) -> Self {
        Normalize3d {
            size_in_world,
            desired_pixel_size,
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn normalize(
    windows: Res<Windows>,
    mut query: QuerySet<(
        QueryState<(&GlobalTransform, &Camera)>,
        QueryState<(&mut Transform, &mut GlobalTransform, &Normalize3d)>,
    )>,
) {
    let (camera_position, camera) = query
        .q0()
        .iter()
        .filter(|(_, cam)| cam.name == Some("camera_3d".to_string()))
        .next()
        .expect("No camera found");
    let camera_position = camera_position.to_owned();
    let view = camera_position.compute_matrix().inverse();
    let camera = Camera {
        window: camera.window,
        projection_matrix: camera.projection_matrix,
        ..Default::default()
    };

    for (mut transform, mut global_transform, normalize) in query.q1().iter_mut() {
        let distance = view.transform_point3(global_transform.translation).z;

        let pixel_end = if let Some(coords) = world_to_screen(
            &camera,
            &windows,
            &GlobalTransform::default(),
            Vec3::new(
                normalize.size_in_world * global_transform.scale.x,
                0.0,
                distance,
            ),
        ) {
            coords
        } else {
            break;
        };
        let pixel_root = if let Some(coords) = world_to_screen(
            &camera,
            &windows,
            &GlobalTransform::default(),
            Vec3::new(0.0, 0.0, distance),
        ) {
            coords
        } else {
            break;
        };
        let actual_pixel_size = pixel_root.distance(pixel_end);
        let required_scale = normalize.desired_pixel_size / actual_pixel_size;
        global_transform.scale *= Vec3::splat(required_scale);
        transform.scale = global_transform.scale;
    }
}

pub fn world_to_screen(
    camera: &Camera,
    windows: &Windows,
    camera_transform: &GlobalTransform,
    world_position: Vec3,
) -> Option<Vec2> {
    let window = windows.get(camera.window)?;
    let window_size = Vec2::new(window.width(), window.height());
    // Build a transform to convert from world to NDC using camera data
    let world_to_ndc: Mat4 = camera.projection_matrix * camera_transform.compute_matrix().inverse();
    let ndc_space_coords: Vec3 = world_to_ndc.project_point3(world_position);
    // NDC z-values outside of 0 < z < 1 are behind the camera and are thus not in screen space
    if ndc_space_coords.z < 0.0 || ndc_space_coords.z > 1.0 || ndc_space_coords.z.is_nan() {
        return None;
    }
    // Once in NDC space, we can discard the z element and rescale x/y to fit the screen
    let screen_space_coords = (ndc_space_coords.truncate() + Vec2::ONE) / 2.0 * window_size;
    Some(screen_space_coords)
}
