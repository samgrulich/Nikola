use glam::{Vec3A, vec3a};

pub mod kernel;
pub mod particle;
pub mod neighborhoods;
pub mod pressure;
pub mod solver;
//
pub use kernel::*;
pub use particle::*;
pub use neighborhoods::*;
pub use pressure::*;
pub use solver::*;
//

pub const REST_DENSITY: f32 = 1000.0;
pub const GRAVITATIONAL_ACCELERATION: Vec3A = vec3a(0.0, -9.81, 0.0);

pub const PARTICLE_RADIUS: f32 = fluid_renderer::PARTICLE_SIZE;
pub const SMOOTHING_LENGHT: f32 = 2.0 * PARTICLE_RADIUS;

