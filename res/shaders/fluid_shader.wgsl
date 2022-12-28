struct Particle {
    @location(0) pos: vec2<f32>,
    @location(1) vel: vec2<f32>,
}

@group(0) @binding(0) var<storage, read_write> particles: array<Particle>;

@compute @workgroup_size(1, 1)
fn main(
    @builtin(global_invocation_id) global_id: vec3<u32>,
) {
    let id = global_id.y * 15u + global_id.x;
    var particle = particles[id];
   
    // velocity update
    particle.vel.y -= 0.0003f;
    for (var i: i32 = 0; i < i32(arrayLength(&particles)); i++) {
        if (i == i32(id)){
            continue;
        }
        
        let other = particles[i];
        let dst = distance(particle.pos, other.pos);

        if (dst <= 0.5f) {
            particle.vel *= -1f;
            break;
        }
    }

    // position update
    particle.pos += particle.vel;

    if (particle.pos.y >= 0f) {
        // todo
        // output should be written into separate buffer
        // these should be after all swapped, because this way parparticles cannot interact properly
        // newton's 3rd law may go missing
        particles[id] = particle;
    }
}
