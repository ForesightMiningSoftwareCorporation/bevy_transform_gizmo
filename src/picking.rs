use bevy::prelude::*;
use bevy_mod_raycast::RaycastSystem;

use crate::{GizmoSettings, TransformGizmoSystem};

pub type GizmoPickSource = bevy_mod_raycast::RaycastSource<GizmoRaycastSet>;
pub type PickableGizmo = bevy_mod_raycast::RaycastMesh<GizmoRaycastSet>;

/// Plugin with all the systems and resources used to raycast against gizmo handles separately from
/// the `bevy_mod_picking` plugin.
pub struct GizmoPickingPlugin;
impl Plugin for GizmoPickingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            (
                update_gizmo_raycast_with_cursor,
                bevy_mod_raycast::build_rays::<GizmoRaycastSet>
                    .in_set(RaycastSystem::BuildRays::<GizmoRaycastSet>),
                bevy_mod_raycast::update_raycast::<GizmoRaycastSet>
                    .in_set(RaycastSystem::UpdateRaycast::<GizmoRaycastSet>),
            )
                .chain()
                .in_set(TransformGizmoSystem::RaycastSet),
        )
        .configure_set(
            TransformGizmoSystem::RaycastSet
                .run_if(|settings: Res<GizmoSettings>| settings.enabled)
                .in_base_set(CoreSet::PreUpdate),
        );
    }
}

#[derive(Reflect, Clone)]
pub struct GizmoRaycastSet;

/// Update the gizmo's raycasting source with the current mouse position.
fn update_gizmo_raycast_with_cursor(
    mut cursor: EventReader<CursorMoved>,
    mut query: Query<&mut GizmoPickSource>,
) {
    for mut pick_source in &mut query.iter_mut() {
        // Grab the most recent cursor event if it exists:
        if let Some(cursor_latest) = cursor.iter().last() {
            pick_source.cast_method =
                bevy_mod_raycast::RaycastMethod::Screenspace(cursor_latest.position);
        }
    }
}
