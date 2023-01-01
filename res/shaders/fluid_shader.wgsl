struct Particle {
    @location(0) pos: vec2<f32>,
    @location(1) vel: vec2<f32>,
}

@group(0) @binding(0) var<storage> ins: array<Particle>;
@group(0) @binding(1) var<storage, read_write> outs: array<Particle>;

@compute @workgroup_size(8, 8)
fn main(
    @builtin(global_invocation_id) global_id: vec3<u32>,
) {
    let id = global_id.y * 4u + global_id.x;
    var particle = ins[id];
   
    // velocity update
    particle.vel.y -= 0.0003f;
    for (var i: i32 = 0; i < i32(arrayLength(&ins)); i++) {
        if (i == i32(id)){
            continue;
        }
        
        let other = ins[i];
        let dst = distance(particle.pos, other.pos);

        if (dst <= 0.5f) {
            particle.pos.y += sign(particle.pos.y - other.pos.y) * 0.001f;
            particle.vel *= -1f;
            break;
        }
    }

    // position update
    particle.pos += particle.vel;

    if (particle.pos.y >= 0f) {
        outs[id] = particle;
    }
}
