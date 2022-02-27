// Vertex shader
struct CameraUniform {
    view_proj: mat4x4<f32>;
};
[[group(1), binding(0)]]
var<uniform> camera: CameraUniform;

struct VertexInput {
    [[location(0)]] position : vec3<f32>;
    [[location(1)]] color : vec3<f32>;
    [[location(2)]] normal : vec3<f32>;
    [[location(3)]] uv : vec2<f32>;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] color : vec3<f32>;
    [[location(1)]] normal : vec3<f32>;
    [[location(2)]] uv : vec2<f32>;
};

[[stage(vertex)]]
fn vs_main(in : VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(in.position, 1.0);
    out.color = in.color;
    out.normal = in.normal;
    out.uv = in.uv;
    return out;
}

[[group(0), binding(0)]]
var t_diffuse: texture_2d<f32>;
[[group(0), binding(1)]]
var s_diffuse: sampler;

 // Fragment shader
[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.uv);
}