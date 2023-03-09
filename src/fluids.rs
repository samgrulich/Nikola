pub mod kernel;
pub mod particle;
pub mod pressure;
pub mod non_pressure;
pub mod neighborhoods;

pub use kernel::*;
pub use particle::*;
pub use pressure::*;
pub use non_pressure::*;
pub use neighborhoods::*;

pub const SMOOTHING_LENGHT: f32 = 2.0;
pub const GRAVITATIONAL_ACCELERATION: f32 = -9.81;
pub const REST_DENSITY: f32 = 1000.0;
