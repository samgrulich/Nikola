use glam::Vec3A;

pub struct ParticleSystem {
    domain_start: [i32],
    domain_end: [i32],
    domain_size: [i32],

    dimensions: u32,

    particle_radius: f32,
    particle_diameter: f32,
    support_radius: f32,

    m_v0: f32,

    grid_size: [u32],
    grid_num: [u32],
    grid_particles_num: [u32],
    grid_particles_num_temp: [u32],
    grid_ids: [u32],
    grid_ids_buffer: [u32],
    grid_ids_new: [u32],

    fluid_particle_num: [u32],
    particle_max_num: [u32],

    x: [[f32]],
    x_0: [[f32]],
    v: [[f32]],
    acceleration: [[f32]],
    m_v: [f32],
    m: [f32],
    density: [f32],
    pressure: [f32],
    color: [Vec3A],
    
    x_buffer: [[f32]],
    x_0_buffer: [[f32]],
    v_buffer: [[f32]],
    acceleration_buffer: [[f32]],
    m_v_buffer: [f32],
    m_buffer: [f32],
    density_buffer: [f32],
    pressure_buffer: [f32],
    color_buffer: [Vec3A],
}

