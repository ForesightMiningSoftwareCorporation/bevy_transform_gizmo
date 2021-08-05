use crate::TransformGizmoPluginConfig;
use bevy::{prelude::*, render::camera::Camera, transform::TransformSystem};

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemLabel)]
pub enum FseNormalizeSystem {
    Normalize,
}

pub struct Ui3dNormalization;
impl Plugin for Ui3dNormalization {
    fn build(&self, app: &mut App) {
        let mut system = normalize
            .system()
            .label(FseNormalizeSystem::Normalize)
            .before(TransformSystem::TransformPropagate);
        if let Some(TransformGizmoPluginConfig {
            run_criteria_producer,
        }) = app.world.get_resource()
        {
            system = system.with_run_criteria(run_criteria_producer());
        }
        app.add_system_to_stage(CoreStage::PostUpdate, system);
    }
}

/// Marker struct that marks entities with meshes that should be scaled relative to the camera.
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
