struct Mesh {
    model: mat4x4<f32>,
    inverse_transpose_model: mat4x4<f32>,
    // 'flags' is a bit field indicating various options. u32 is 32 bits so we have up to 32 options.
    flags: u32,
};

struct View {
    view_proj: mat4x4<f32>,
    inverse_view: mat4x4<f32>,
    projection: mat4x4<f32>,
    world_position: vec3<f32>,
    near: f32,
    far: f32,
    width: f32,
    height: f32,
};

@group(2) @binding(0)
var<uniform> mesh: Mesh;

@group(0) @binding(0)
var<uniform> view: View;

struct GizmoMaterial {
    color: vec4<f32>,
};

@group(1) @binding(0)
var<uniform> material: GizmoMaterial;

struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    let world_position = mesh.model * vec4<f32>(vertex.position, 1.0);
    var out: VertexOutput;
    var modified_clip = view.view_proj * world_position;
    out.clip_position = modified_clip;
    return out;
}

@fragment
fn fragment() -> @location(0) vec4<f32> {
    return material.color;
}