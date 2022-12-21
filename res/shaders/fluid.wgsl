// a_i = f_i / rho_i

struct Particle {
    @location(0) density: f32,
    @location(1) position: f32,
    @location(2) velocity: f32,
};

@group(0) @binding(0) var<storage> particles: array<Particle>;
@group(0) @binding(1) var<storage> time_step: f32;

const H = 10f;

@compute @workgroup_size(8, 8)
fn step(
    @builtin(global_invocation_id) global_id: vec3<u32>
) {
    let local_particle = particles[global_id];
    let particles_len = arrayLength(particles); // supposed to be ptr
    var distances = array<f32>;
    var distances_len = 0u;

    // compute distances
    for (var i: i32 = 0; i < i32(particles_len); i++)
    {
        let particle = particles[i];
        let dist = distance(particle.position, local_particle.position);

        if (dist <= H) {
            distances[distance_len] = dist;
            distance_len = distance_len + 1u;
        }
    }

    // compute density
    let density = 1;

    // compute forces
        // pressure
        // viscosity
        // surface tension

    // compute color gradient - surface particles
    
    // color surface tension

    // compute acceleration
    // add acceleration * time_step to the velocity of particle
    // add the velocity to the position
    // write new parctile values to the buffer

    // render 
}

@compute @workgroup_size(8, 8)
fn render(
