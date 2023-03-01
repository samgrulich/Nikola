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

const SMOOTHING_LENGHT: f32 = 2.0;
const GRAVITATIONAL_ACCELERATION: f32 = -9.81;
