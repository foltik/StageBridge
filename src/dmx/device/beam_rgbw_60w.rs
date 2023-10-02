//! 60W RGBW moving head
//!
//! https://www.amazon.com/gp/product/B089QGPJ2L
//! https://www.aliexpress.com/w/wholesale-Beam-60W-LED-Moving-Head-RGBW-4-IN-1-Stage-Lightin.html

use crate::dmx::Device;
use crate::num::Float;
use crate::color::Rgbw;

#[derive(Clone, Copy, Debug)]
pub struct Beam {
    pub mode: BeamMode,
    pub ring: BeamRing,

    pub pitch: f64,
    pub yaw: f64,
    pub speed: f64,

    pub color: Rgbw,
    pub alpha: f64,
}

#[derive(Clone, Copy, Debug)]
pub enum BeamMode {
    Manual,
    ColorCycle,
    Auto,
}

#[derive(Clone, Copy, Debug)]
pub enum BeamRing {
    Off,

    Red,
    Green,
    Blue,
    Yellow,
    Purple,
    Teal,
    White,

    RedYellow,
    RedPurple,
    RedWhite,

    GreenYellow,
    GreenBlue,
    GreenWhite,

    BluePurple,
    BlueTeal,
    BlueWhite,

    Cycle,
    Raw(u8),
}


impl Device for Beam {
    fn channels(&self) -> usize { 15 }

    fn encode(&self, buf: &mut [u8]) {
        buf[0] = self.yaw.byte();
        // buf[0] = (self.yaw * (2.0 / 3.0)).byte();
        buf[0] = self.yaw.lerp((1.0/3.0)..1.0).byte();
        // buf[1]: yaw fine
        buf[2] = self.pitch.byte();
        // buf[3]: pitch fine
        buf[4] = (1.0 - self.speed).byte();
        buf[5] = self.alpha.byte();
        // buf[6]: strobe
        buf[7] = self.color.r().byte();
        buf[8] = self.color.g().byte();
        buf[9] = self.color.b().byte();
        buf[10] = self.color.w().byte();
        // buf[11]: color preset
        buf[12] = self.mode.byte();
        // buf[13]: auto pitch/yaw, reset
        buf[14] = self.ring.byte();
    }
}

impl BeamRing {
    pub fn byte(&self) -> u8 {
        match self {
            BeamRing::Off => 0,

            BeamRing::Red => 4,
            BeamRing::Green => 22,
            BeamRing::Blue => 36,
            BeamRing::Yellow => 56,
            BeamRing::Purple => 74,
            BeamRing::Teal => 84,
            BeamRing::White => 104,

            BeamRing::RedYellow => 116,
            BeamRing::RedPurple => 128,
            BeamRing::RedWhite => 140,

            BeamRing::GreenYellow => 156,
            BeamRing::GreenBlue => 176,
            BeamRing::GreenWhite => 192,

            BeamRing::BluePurple => 206,
            BeamRing::BlueTeal => 216,
            BeamRing::BlueWhite => 242,

            BeamRing::Cycle => 248,
            BeamRing::Raw(i) => *i,
        }
    }
}

impl BeamMode {
    pub fn byte(&self) -> u8 {
        match self {
            BeamMode::Manual => 0,
            BeamMode::ColorCycle => 159,
            BeamMode::Auto => 60,
        }
    }
}

impl Default for Beam {
    fn default() -> Self {
        Self {
            mode: BeamMode::Manual,
            ring: BeamRing::Off,

            pitch: 0.0,
            yaw: 2.0 / 3.0,
            speed: 1.0,

            color: Rgbw::white(),
            alpha: 1.0,
        }
    }
}
