//! 12x3W RGBW par light
//!
//! https://www.aliexpress.com/w/wholesale-12x3w-rgbw-dmx-led-par-light.html

use crate::dmx::Device;
use crate::num::Float;
use crate::color::Rgbw;

pub struct Par {
    pub color: Rgbw,
    pub alpha: f64,
}

impl Device for Par {
    fn channels(&self) -> usize { 8 }

    fn encode(&self, buf: &mut [u8]) {
        // buf[0]: ?
        // buf[1]: ?
        // buf[2]: ?
        buf[3] = self.alpha.byte();
        buf[4] = self.color.r().byte();
        buf[5] = self.color.g().byte();
        buf[6] = self.color.b().byte();
        buf[7] = self.color.w().byte();
    }
}

impl Default for Par {
    fn default() -> Self {
        Self {
            color: Rgbw::off(),
            alpha: 1.0,
        }
    }
}
