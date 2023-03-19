use std::time::Duration;

// pub mod particles;
// pub mod simulation;
//
// pub use particles::*;
// pub use simulation::*;

pub const DIMENSIONS: (i32, i32, i32) = (4, 2, 4);
pub const PARTICLE_RADIUS: f32 = 0.1;
pub const PARTICLE_OFFSET: f32 = 0.1;
pub const FLUID_OFFSET: f32 = 10.0;
pub const FLUID_TIMESTEP: Duration = Duration::from_millis(600);
