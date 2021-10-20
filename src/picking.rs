use bevy::prelude::*;
use bevy_mod_raycast::RaycastSystem;

pub type GizmoPickSource = bevy_mod_raycast::RayCastSource<GizmoRaycastSet>;
pub type PickableGizmo = bevy_mod_raycast::RayCastMesh<GizmoRaycastSet>;

/// Plugin with all the systems and resources used to raycast against gizmo handles separately from
/// the `bevy_mod_picking` plugin.
pub struct GizmoPickingPlugin;
impl Plugin for GizmoPickingPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(
            CoreStage::PreUpdate,
            bevy_mod_raycast::build_rays::<GizmoRaycastSet>.label(RaycastSystem::BuildRays),
        )
        .add_system_to_stage(
            CoreStage::PreUpdate,
            bevy_mod_raycast::update_raycast::<GizmoRaycastSet>
                .label(RaycastSystem::UpdateRaycast)
                .after(RaycastSystem::BuildRays),
        )
        .add_system_to_stage(
            CoreStage::PreUpdate,
            update_gizmo_raycast_with_cursor.before(RaycastSystem::BuildRays),
        );
    }
}

pub struct GizmoRaycastSet;

/// Update the gizmo's raycasting source with the current mouse position.
fn update_gizmo_raycast_with_cursor(
    mut cursor: EventReader<CursorMoved>,
    mut query: Query<&mut bevy_mod_raycast::RayCastSource<GizmoRaycastSet>>,
) {
    for mut pick_source in &mut query.iter_mut() {
        // Grab the most recent cursor event if it exists:
        if let Some(cursor_latest) = cursor.iter().last() {
            pick_source.cast_method =
                bevy_mod_raycast::RayCastMethod::Screenspace(cursor_latest.position);
        }
    }
}
