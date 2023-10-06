// #![feature(associated_type_defaults)]
// #![feature(associated_type_bounds)]
// #![feature(async_closure)]
// #![feature(trait_upcasting)]
// #![feature(generic_const_exprs)]
#![feature(lazy_cell)]

#![allow(incomplete_features)]
#![allow(clippy::module_inception)]
#![allow(clippy::eq_op)]

#[cfg(feature = "midi")]
pub mod midi;

// #[cfg(feature = "osc")]
// pub mod osc;

#[cfg(feature = "e131")]
pub mod e131;

#[cfg(feature = "dmx")]
pub mod dmx;

pub mod color;

pub mod util;
pub mod num;
