// Vertex shader

[[block]]
struct Uniforms {
    view_pos: vec4<f32>;
    view_proj: mat4x4<f32>;
};
[[group(0), binding(0)]]
var<uniform> uniforms: Uniforms;

struct VertexInput {
    [[location(0)]] position: vec3<f32>;
};
struct InstanceInput {
    [[location(5)]] model_matrix_0: vec4<f32>;
    [[location(6)]] model_matrix_1: vec4<f32>;
    [[location(7)]] model_matrix_2: vec4<f32>;
    [[location(8)]] model_matrix_3: vec4<f32>;
    [[location(9)]] color: vec4<f32>;
    [[location(10)]] attrs: i32;
    [[builtin(vertex_index)]] vertex_index: u32;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] color: vec4<f32>;
    [[location(1)]] attrs: i32;
    [[location(2)]] age: f32;
};

[[stage(vertex)]]
fn main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );

    let world_position = vec4<f32>(model.position, 1.0);

    var out: VertexOutput;
    out.clip_position = uniforms.view_proj * world_position;
    out.color = instance.color;
    out.attrs = instance.attrs;
    out.age = f32(instance.vertex_index);
    return out;
}

// Fragment shader

[[stage(fragment)]]
fn main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let enabled = (in.attrs & 1) > 0;
    let weight = exp(-0.05 * in.age);

    if (!enabled || weight < 0.01) {
        discard;
    }

    return weight * in.color;
}