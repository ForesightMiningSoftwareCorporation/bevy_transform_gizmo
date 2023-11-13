use bevy::{
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    reflect::{TypePath},
    render::{
        mesh::MeshVertexBufferLayout,
        render_resource::{
            AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
        },
    },
};

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct GizmoMaterial {
    #[uniform(0)]
    pub color: Color,
}

impl From<Color> for GizmoMaterial {
    fn from(color: Color) -> Self {
        GizmoMaterial { color }
    }
}

impl Material for GizmoMaterial {
    fn vertex_shader() -> ShaderRef {
        "gizmo_material.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "gizmo_material.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Opaque
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = None;
        Ok(())
    }
}
