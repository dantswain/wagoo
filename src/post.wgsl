// Vertex shader
[[block]]
struct Uniforms {
    flags: i32;
};
[[group(1), binding(0)]]
var<uniform> uniforms: Uniforms;

struct VertexInput {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] tex_coords: vec2<f32>;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] tex_coords: vec2<f32>;
};

[[stage(vertex)]]
fn main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.clip_position = vec4<f32>(model.position, 1.0);
    return out;
}

// Fragment shader

[[group(0), binding(0)]]
var t_diffuse: texture_2d<f32>;
[[group(0), binding(1)]]
var s_diffuse: sampler;


[[stage(fragment)]]
fn main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let dims = textureDimensions(t_diffuse);
    let dx = 2.0 / f32(dims.x);
    let dy = 2.0 / f32(dims.y);

    let enabled = (uniforms.flags & 1) > 0;
    let horizontal = (uniforms.flags & 2) > 0;

    var result: vec4<f32>;
    if (!enabled) {
        result = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    } else {
        result = 0.3333 * textureSample(t_diffuse, s_diffuse, in.tex_coords);
        if (horizontal) {
            result = result + 0.3333 * textureSample(t_diffuse, s_diffuse, in.tex_coords - vec2<f32>(dx, 0.0));
            result = result + 0.3333 * textureSample(t_diffuse, s_diffuse, in.tex_coords + vec2<f32>(dx, 0.0));
        } else {
            result = result + 0.3333 * textureSample(t_diffuse, s_diffuse, in.tex_coords - vec2<f32>(0.0, dy));
            result = result + 0.3333 * textureSample(t_diffuse, s_diffuse, in.tex_coords + vec2<f32>(0.0, dy));
        }
    }
    return result;
}