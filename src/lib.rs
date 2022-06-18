#![feature(associated_type_defaults)]
#![feature(associated_type_bounds)]

#![feature(async_closure)]

#![feature(trait_upcasting)]
#![allow(incomplete_features)]

#[cfg(feature = "midi")]
pub mod midi;

#[cfg(feature = "osc")]
pub mod osc;

#[cfg(feature = "e131")]
pub mod e131;

pub mod util;