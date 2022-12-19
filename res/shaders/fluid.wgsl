// a_i = f_i / rho_i

struct Particle {
    @location(0) density: f32,
    @location(1) position: f32,
    @location(2) velocity: f32,
};

@group(0) @binding(0) var<storage> particles: array<Particle>;
@group(0) @binding(1) var<storage> time_step: f32;

@compute @workgroup_size(8, 8)
fn main(
    @builtin(global_invocation_id) global_id: vec3<u32>
) {
    let local_particle = particles[global_id];
    let particles_len = arrayLength(particles); // supposed to be ptr
    var distances = array<f32, particles_len>;

    for particle in particles 
    {

    }
    // compute distances fn: dinstance()

    // compute density

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
