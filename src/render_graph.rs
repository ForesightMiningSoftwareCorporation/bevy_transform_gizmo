use bevy::{
    prelude::*,
    render::{
        pass::{
            LoadOp, Operations, PassDescriptor, RenderPassDepthStencilAttachment, TextureAttachment,
        },
        render_graph,
        render_graph::base,
    },
};
#[derive(Default)]
pub struct GizmoPass;

pub mod node {
    pub const GIZMO_PASS: &str = "gizmo_pass";
}

pub fn add_gizmo_graph(world: &mut World) {
    let world = world.cell();
    let mut graph = world
        .get_resource_mut::<render_graph::RenderGraph>()
        .unwrap();
    let msaa = world.get_resource::<Msaa>().unwrap();

    let mut gizmo_pass_node = render_graph::PassNode::<&GizmoPass>::new(PassDescriptor {
        color_attachments: vec![msaa.color_attachment(
            TextureAttachment::Input("color_attachment".to_string()),
            TextureAttachment::Input("color_resolve_target".to_string()),
            Operations {
                load: LoadOp::Load,
                store: true,
            },
        )],
        depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
            attachment: TextureAttachment::Input("depth".to_string()),
            depth_ops: Some(Operations {
                load: LoadOp::Clear(1.0),
                store: true,
            }),
            stencil_ops: None,
        }),
        sample_count: msaa.samples.clone(),
    });
    gizmo_pass_node.add_camera(base::camera::CAMERA_3D);

    graph.add_node(node::GIZMO_PASS, gizmo_pass_node);

    graph
        .add_slot_edge(
            base::node::PRIMARY_SWAP_CHAIN,
            render_graph::WindowSwapChainNode::OUT_TEXTURE,
            node::GIZMO_PASS,
            if msaa.samples > 1 {
                "color_resolve_target"
            } else {
                "color_attachment"
            },
        )
        .unwrap();

    graph
        .add_slot_edge(
            base::node::MAIN_DEPTH_TEXTURE,
            render_graph::WindowTextureNode::OUT_TEXTURE,
            node::GIZMO_PASS,
            "depth",
        )
        .unwrap();

    if msaa.samples > 1 {
        graph
            .add_slot_edge(
                base::node::MAIN_SAMPLED_COLOR_ATTACHMENT,
                render_graph::WindowSwapChainNode::OUT_TEXTURE,
                node::GIZMO_PASS,
                "color_attachment",
            )
            .unwrap();
    }

    // ensure gizmo pass runs after main pass
    graph
        .add_node_edge(base::node::MAIN_PASS, node::GIZMO_PASS)
        .unwrap();
}
