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
pub struct Normalize3d {
    pub scale: f32,
}
impl Default for Normalize3d {
    fn default() -> Self {
        Self { scale: 1.0 }
    }
}

#[allow(clippy::type_complexity)]
pub fn normalize(
    mut query: QuerySet<(
        QueryState<(&Transform, &Camera)>,
        QueryState<(&mut Transform, &Normalize3d)>,
    )>,
) {
    // TODO: can be improved by manually specifying the active camera to normalize against. The
    // majority of cases will only use a single camera for this viewer, so this is sufficient.
    let (camera_position, camera) = query.q0().get_single().expect("Not exactly one camera");
    let camera_position = camera_position.to_owned();

    let projection = camera.projection_matrix;
    let z_0 = projection.project_point3(Vec3::new(1.0, 0.0, -1.0)).x;
    let z_1 = projection.project_point3(Vec3::new(1.0, 0.0, -2.0)).x;
    let projection_scale = (z_0 / z_1).powf(1.0 / 3.0);

    let view = camera_position.compute_matrix().inverse();

    for (mut transform, normalize) in query.q1().iter_mut() {
        let distance = view.transform_point3(transform.translation).z.abs();
        transform.scale =
            Vec3::splat((1.0 + (distance * projection_scale) - distance) * 0.2 * normalize.scale);
    }
}

/*let z_0 = camera
    .projection_matrix
    .project_point3(Vec3::new(1.0, 0.0, 1.0))
    .x;
let z_1 = camera
    .projection_matrix
    .project_point3(Vec3::new(1.0, 0.0, 2.0))
    .x;
let projection_scale = ((z_1 / z_0).powi(2) * 3.0).sqrt();

let view = camera_position.compute_matrix().inverse();

for (mut transform, normalize) in query.q1().iter_mut() {
    let distance = view.transform_point3(transform.translation).z.abs();
    transform.scale =
        Vec3::splat((1.0 + distance - (distance * projection_scale)) * 0.2 * normalize.scale);
}*/
