use fluid_renderer::Instance;
use glam::Vec3A;

/// Configuration struct used for initialization of particle system
pub struct Config {
    /// Starting point of domain
    pub domain_start: Vec3A,
    /// Ending point of domain
    pub domain_end: Vec3A,

    /// Radius of particle
    pub particle_radius: f32,

    /// Amount of particles
    pub particle_num: usize,
    /// Rest density 
    pub density_0: f32,
    /// Initial positions 
    pub x: Vec<Vec3A>,
    /// Initial velocities
    pub v: Vec<Vec3A>,
    /// Color of each particle
    pub color: Vec<Vec3A>,
}

impl Config {
    /// Creates Configuration struct from vec of instances
    ///
    /// # Arguments 
    /// * `domain_start` - Starting point of domain
    /// * `domain_end` - Ending point of domain
    /// * `particle_radius` - Radius of particle
    /// * `density_0` - Rest density
    /// * `instances` - Instances :)
    ///
    /// # Returns
    /// new configuration struct
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
