struct Particle {
    @location(0) position: vec2<f32>,
    @location(1) velocity: vec2<f32>,
    @location(2) mass: f32,
    @location(3) density: f32,
}

@group(0) @binding(0) var out_texture: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(1) var<storage, read> particles: array<Particle>;

@compute @workgroup_size(1, 1)
fn main(
    @builtin(global_invocation_id) global_id: vec3<u32>,
) {
    let dimensions = textureDimensions(out_texture);
    let pixel_coords = vec2<i32>(global_id.xy);

    if (pixel_coords.x >= dimensions.x || pixel_coords.y >= dimensions.y) {
        return;
    }

    let color = vec3(
        vec2<f32>(global_id.xy) / vec2<f32>(dimensions / 9) - vec2(3.2f, 2f),
        0f
    );

    let position = vec2<f32>(color.xy);
    let particles_len = i32(arrayLength(&particles));
    var closest: f32 = distance(particles[0].position, position);
    var closest_idx: f32 = 0f;

    for (var i: i32 = 0; i < particles_len; i++ ) {
        let dist = distance(particles[i].position, position);

        if (dist < closest) {
            closest = dist;
            closest_idx = f32(i);
        }
    }

    let dst = 1f - step(0.5f, closest);
    let particle = particles[i32(closest_idx)];
//    var color = vec3(
//        dst * 0.7f * closest_idx / 16f, 
//        closest_idx / 16f, 
//        dst
//    );

//    var color = vec3(abs(particle.velocity) * 20f, 0f); 
    var color = vec3(particle.density - 100f); 

    if (dst <= 0f) {
        color = vec3(0.8f);
    }

    textureStore(out_texture, pixel_coords, vec4(color, 1f));
}
