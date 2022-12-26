struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
}

struct VertOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex 
fn vert_main(
    model: Vertex
) -> VertOut {
    var out: VertOut;

    out.clip_position = vec4(model.position, 1f);
    out.uv = model.uv;

    return out;
}

@group(0) @binding(0) var t_diffuse: texture_2d<f32>;
@group(0) @binding(1) var s_diffuse: sampler;

@fragment
fn frag_main(
    in: VertOut
) -> @location(0) vec4<f32> {
    let diffuse = textureSample(t_diffuse, s_diffuse, in.uv);
    let color = diffuse;

    return color;
}
