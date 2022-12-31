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
    model: Vertex,
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertOut {
    var out: VertOut;

    out.clip_position = vec4(model.position, 1f);
    out.uv = model.uv;

    return out;
}

// @group(0) @binding(0) var s_diffuse: sampler;
// @group(0) @binding(1) var t_diffuse: texture_2d<f32>;

@fragment
fn frag_main(
    in: VertOut
) -> @location(0) vec4<f32> {
    // let diffuse = textureSample(t_diffuse, s_diffuse, in.uv);
    // let color = diffuse;
    let color = vec4<f32>(1f, 1f, 0f, 1f);

    return color;
}
