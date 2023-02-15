pub mod kernel;
pub mod particle;
pub mod pressure;
pub mod neighborhoods;

pub use kernel::*;
pub use particle::*;
pub use pressure::*;
pub mod neighborhoods::*;

const SMOOTHING_LENGHT: f32 = 2.0;
