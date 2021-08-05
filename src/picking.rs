use bevy::prelude::*;

use crate::{TransformGizmo, TransformGizmoPluginConfig};

pub type GizmoPickSource = bevy_mod_raycast::RayCastSource<GizmoRaycastSet>;
pub type PickableGizmo = bevy_mod_raycast::RayCastMesh<GizmoRaycastSet>;

/// Plugin with all the systems and resources used to raycast against gizmo handles separately from
/// the `bevy_mod_picking` plugin.
pub struct GizmoPickingPlugin;
impl Plugin for GizmoPickingPlugin {
    fn build(&self, app: &mut App) {
        let mut build_rays = bevy_mod_raycast::build_rays::<GizmoRaycastSet>
            .system()
            .label(bevy_mod_raycast::RaycastSystem::BuildRays);
        let mut update_raycast = bevy_mod_raycast::update_raycast::<GizmoRaycastSet>
            .system()
            .label(bevy_mod_raycast::RaycastSystem::UpdateRaycast)
            .after(bevy_mod_raycast::RaycastSystem::BuildRays);
        let mut update_gizmo = update_gizmo_raycast_with_cursor
            .system()
            .before(bevy_mod_raycast::RaycastSystem::BuildRays);
        let mut disable_mesh_picking = disable_mesh_picking_during_gizmo_hover
            .system()
            .before(bevy_mod_picking::PickingSystem::Focus)
            .after(bevy_mod_raycast::RaycastSystem::UpdateRaycast);
        if let Some(TransformGizmoPluginConfig {
            run_criteria_producer,
        }) = app.world.get_resource()
        {
            build_rays = build_rays.with_run_criteria(run_criteria_producer());
            update_raycast = update_raycast.with_run_criteria(run_criteria_producer());
            update_gizmo = update_gizmo.with_run_criteria(run_criteria_producer());
            disable_mesh_picking = disable_mesh_picking.with_run_criteria(run_criteria_producer());
        }
        app.init_resource::<bevy_mod_raycast::PluginState<GizmoRaycastSet>>()
            .add_system_to_stage(CoreStage::PreUpdate, build_rays)
            .add_system_to_stage(CoreStage::PreUpdate, update_raycast)
            .add_system_to_stage(CoreStage::PreUpdate, update_gizmo)
            .add_system_to_stage(CoreStage::PreUpdate, disable_mesh_picking);
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

/// Disable the picking plugin when the mouse is over one of the gizmo handles.
fn disable_mesh_picking_during_gizmo_hover(
    mut picking_state: ResMut<bevy_mod_picking::PickingPluginState>,
    query: Query<&bevy_mod_raycast::RayCastSource<GizmoRaycastSet>>,
    gizmo_query: Query<&TransformGizmo>,
) {
    let not_hovering_gizmo = if let Some(source) = query.iter().last() {
        source.intersect_top().is_none()
    } else {
        true
    };
    let gizmo_inactive = if let Some(gizmo) = gizmo_query.iter().last() {
        gizmo.current_interaction().is_none()
    } else {
        return;
    };
    // Set the picking state based on current user interaction state
    picking_state.enabled = gizmo_inactive && not_hovering_gizmo;
}
