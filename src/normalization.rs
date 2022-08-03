use bevy::{prelude::*, render::camera::Camera, transform::transform_propagate_system};
use bevy_mod_picking::PickingCamera;

use crate::GizmoSystemsEnabledCriteria;

pub struct Ui3dNormalization;
impl Plugin for Ui3dNormalization {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(
            CoreStage::PostUpdate,
            normalize
                .with_run_criteria(GizmoSystemsEnabledCriteria)
                .after(transform_propagate_system)
                .after(crate::place_gizmo),
        );
    }
}

/// Marker struct that marks entities with meshes that should be scaled relative to the camera.
#[derive(Component, Debug)]
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
    images: Res<Assets<Image>>,
    mut query: ParamSet<(
        Query<(&GlobalTransform, &Camera), With<PickingCamera>>,
        Query<(&mut Transform, &mut GlobalTransform, &Normalize3d)>,
    )>,
) {
    // TODO: can be improved by manually specifying the active camera to normalize against. The
    // majority of cases will only use a single camera for this viewer, so this is sufficient.
    let (camera_position, camera) = if let Ok((camera_position, camera)) = query.p0().get_single() {
        (camera_position.to_owned(), camera.to_owned())
    } else {
        error!("More than one picking camera");
        return;
    };
    let view = camera_position.compute_matrix().inverse();

    for (mut transform, mut global_transform, normalize) in query.p1().iter_mut() {
        let distance = view.transform_point3(global_transform.translation).z;

        let pixel_end = if let Some(coords) = Camera::world_to_screen(
            &camera,
            &windows,
            &images,
            &GlobalTransform::default(),
            Vec3::new(
                normalize.size_in_world * global_transform.scale.x,
                0.0,
                distance,
            ),
        ) {
            coords
        } else {
            continue;
        };
        let pixel_root = if let Some(coords) = Camera::world_to_screen(
            &camera,
            &windows,
            &images,
            &GlobalTransform::default(),
            Vec3::new(0.0, 0.0, distance),
        ) {
            coords
        } else {
            continue;
        };
        let actual_pixel_size = pixel_root.distance(pixel_end);
        let required_scale = normalize.desired_pixel_size / actual_pixel_size;
        global_transform.scale *= Vec3::splat(required_scale);
        transform.scale = global_transform.scale;
    }
}
