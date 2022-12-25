struct Vertex {
    @location(0) position: vec3<f32>,
}

struct VertOut {
    @builtin(position) clip_position: vec4<f32>,
}

@vertex 
fn vert_main(
    model: Vertex
) -> VertOut {
    var out: VertOut;

    out.clip_position = vec4(model.position, 1f);

    return out;
}

@fragment
fn frag_main(
    in: VertOut
) -> @location(0) vec4<f32> {
    let color = vec3(1f, 1f, 0f);

    return vec4(color, 1f);
}
