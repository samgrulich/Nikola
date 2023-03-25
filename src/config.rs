use fluid_renderer::Instance;
use glam::Vec3A;


pub struct Config {
    pub domain_start: Vec3A,
    pub domain_end: Vec3A,

    pub particle_radius: f32,

    pub particle_num: usize,
    pub density_0: f32,
    pub x: Vec<Vec3A>,
    pub v: Vec<Vec3A>,
    pub color: Vec<Vec3A>,
}

impl Config {
    pub fn from_instances(
        domain_start: Vec3A, 
        domain_end: Vec3A,
        particle_radius: f32,
        density_0: f32,
        instances: &Vec<Instance>,
    ) -> Self {
        Config { 
            domain_start, 
            domain_end, 
            particle_radius, 
            particle_num: instances.len(), 
            density_0, 
            x: instances.iter().map(|instance| instance.position.into()).collect(), 
            v: vec![Vec3A::ZERO; instances.len()], 
            color: instances.iter().map(|instance| instance.color.into()).collect()
        }
    }
}
