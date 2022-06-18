use std::{time::Duration};

use fraction::Ratio;

use derive_more::{Add, Sub, Mul, Div};

#[derive(Copy, Clone, PartialEq, Eq, Add, Sub, Mul, Div, Debug)]
pub struct Beats(Ratio<u32>);
impl Beats {
    pub fn duration(&self, bpm: f32) -> Duration {
        let beats = *self.0.numer() as f32 / *self.0.denom() as f32;
        Duration::from_secs_f32(beats * (bpm / 60.0))
    }

    pub fn n(num: u32) -> Self {
        Self(Ratio::new(num, 1))
    }

    pub fn frac(num: u32, denom: u32) -> Self {
        Self(Ratio::new(num, denom))
    }

    pub fn zero() -> Self {
        Self(Ratio::new(0, 1))
    }
}


#[derive(Copy, Clone, Add, Sub, Mul, Div, Debug)]
pub struct Delay {
    time: Duration,
    beats: Beats,
}

impl Delay {
    pub fn duration(&self, bpm: f32) -> Duration {
        self.time + self.beats.duration(bpm)
    }

    pub fn zero() -> Self {
        Self {
            time: Duration::ZERO,
            beats: Beats::zero(),
        }
    }

    pub fn is_zero(&self) -> bool {
        self.time == Duration::ZERO && self.beats == Beats::zero()
    }
}

impl From<Duration> for Delay {
    fn from(d: Duration) -> Self {
        Self {
            time: d,
            beats: Beats::zero()
        }
    }
}

impl From<Beats> for Delay {
    fn from(b: Beats) -> Self {
        Self {
            time: Duration::ZERO,
            beats: b,
        }
    }
}