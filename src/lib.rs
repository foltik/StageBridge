// #![feature(associated_type_defaults)]
// #![feature(associated_type_bounds)]
// #![feature(async_closure)]
// #![feature(trait_upcasting)]
// #![feature(generic_const_exprs)]

#![allow(incomplete_features)]
#![allow(clippy::module_inception)]
#![allow(clippy::eq_op)]

#[cfg(feature = "dmx")]
pub mod dmx;
#[cfg(feature = "e131")]
pub mod e131;
#[cfg(feature = "midi")]
pub mod midi;
#[cfg(feature = "osc")]
pub mod osc;

pub mod color;
pub mod num;

/// A set of common traits and types. Bring in scope with `use prelude::*`.
pub mod prelude {
    pub use crate::color::{Rgb, Rgbw};
    pub use crate::num::{Byte, Ease, Interp, Range};
}
