mod byte;
mod ease;
mod interp;
mod range;

pub use byte::Byte;
pub use ease::Ease;
pub use interp::Interp;
pub use range::Range;

pub use std::f64::consts::TAU;

/// τ/2, a half-circle.
pub const TAU_2: f64 = TAU / 2.0;

/// τ/4, a quarter-circle.
pub const TAU_4: f64 = TAU / 4.0;
