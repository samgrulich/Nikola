use glam::Vec3A;

pub struct Config {
    pub domain_start: Vec3A,
    pub domain_end: Vec3A,

    pub dimensions: u32,

    pub particle_radius: f32,

    pub m_v0: f32,

    pub grid_size: Vec3A,
    pub particle_num: usize,
}

