use bevy::{prelude::*, render::camera::Camera, transform::TransformSystem};

use crate::GizmoSystemsEnabledCriteria;

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
                .before(TransformSystem::TransformPropagate),
        );
    }
}

/// Marker struct that marks entities with meshes that should be scaled relative to the camera.
#[derive(Component)]
pub struct Normalize3d;

#[allow(clippy::type_complexity)]
pub fn normalize(
    camera_query: Query<&GlobalTransform, With<Camera>>,
    mut normalize_query: Query<&mut Transform, With<Normalize3d>>,
) {
    // TODO: can be improved by manually specifying the active camera to normalize against. The
    // majority of cases will only use a single camera for this viewer, so this is sufficient.
    let camera_position = camera_query
        .iter()
        .last()
        .cloned()
        .expect("No camera present in scene");

    for mut transform in normalize_query.iter_mut() {
        let distance = -camera_position
            .compute_matrix()
            .inverse()
            .transform_point3(transform.translation)
            .z;

        transform.scale = Vec3::splat(distance / 12.0);
    }
}
