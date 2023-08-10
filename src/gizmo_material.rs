use bevy::{
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    reflect::{TypePath, TypeUuid},
    render::{
        mesh::MeshVertexBufferLayout,
        render_resource::{
            AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
        },
    },
};

pub const GIZMO_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 13953800272683943019);

#[derive(Debug, Clone, Default, TypeUuid, TypePath, AsBindGroup)]
#[uuid = "0cf245a7-ce7a-4473-821c-111e6f359193"]
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
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Handle(GIZMO_SHADER_HANDLE.typed())
    }

    fn vertex_shader() -> ShaderRef {
        ShaderRef::Handle(GIZMO_SHADER_HANDLE.typed())
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
