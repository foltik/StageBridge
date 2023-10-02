//! 18W RGB light bar
//!
//! https://www.amazon.com/gp/product/B0045EP4WG

use crate::color::Rgb;
use crate::dmx::Device;
use crate::num::Float;

#[derive(Default, Clone, Copy, Debug)]
pub struct Bar {
    pub color: Rgb,
    pub alpha: f64,
}

impl Device for Bar {
    fn channels(&self) -> usize { 7 }

    fn encode(&self, buf: &mut [u8]) {
        buf[0] = self.color.r().byte();
        buf[1] = self.color.g().byte();
        buf[2] = self.color.b().byte();
        // buf[3]: preset colors
        // buf[4]: strobe
        // buf[5]: mode
        buf[6] = self.alpha.byte();
    }
}

// #[derive(Default, Clone, Copy, Debug)]
// pub enum BarMode {
//     #[default]
//     Manual,
//     ColorCycle,
//     Auto,
// }

// impl BarMode {
//     pub fn byte(&self) -> u8 {
//         match self {
//             BarMode::Manual => 0,
//             BarMode::ColorCycle => 159,
//             BarMode::Auto => 60,
//         }
//     }
// }
