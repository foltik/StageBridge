use std::f32::consts::PI;

use super::Range;

pub trait Float: Sized {
    /// 0 below the threshold and 1 above it
    fn step(self, threshold: Self) -> Self;

    /// Euclidean remainder (proper float mod)
    fn fmod(self, v: Self) -> Self;
    /// Interpolate self in 0..1 onto another range
    fn lerp<R: Into<Range>>(self, onto: R) -> Self;
    /// Interpolate self from a range onto 0..1
    fn ilerp<R: Into<Range>>(self, from: R) -> Self;
    /// Map self from a range onto another range
    fn map<R0: Into<Range>, R1: Into<Range>>(self, from: R0, onto: R1) -> Self {
        self.ilerp(from).lerp(onto)
    }
    /// Add `fr` of a periodic `pd` to self
    fn phase(self, pd: Self, fr: Self) -> Self;

    /// Invert self in range
    fn invert<R: Into<Range>>(self, from: R) -> Self {
        let range = from.into();
        self.map(range, range.invert())
    }
    /// Invert self in 0..1
    fn inv(self) -> Self {
        self.invert(0..1)
    }

    /// Project self onto a line, y=mx+b style
    fn line(self, slope: Self, intercept: Self) -> Self;
    /// Project self in 0..1 onto 0..[0.5-amt/2, 0.5+amt/2]..1
    /// https://www.desmos.com/calculator/i6cdluzrj4
    fn cover(self, amt: Self) -> Self;

    /// sin(ish), but domain and range are 0..1
    /// https://www.desmos.com/calculator/c8xbmebyiy
    fn fsin(self, pd: Self) -> Self;
    /// cos(ish), but domain and range are 0..1
    /// https://www.desmos.com/calculator/7efgxmnpoe
    fn fcos(self, pd: Self) -> Self;
    /// Triangle wave
    /// https://www.desmos.com/calculator/psso6ibqq7
    fn tri(self, pd: Self) -> Self;
    /// Ramp (saw wave)
    /// https://www.desmos.com/calculator/v4dlv296h3
    fn ramp(self, pd: Self) -> Self;
    /// Square wave
    /// https://www.desmos.com/calculator/fsfuxn4xvg
    fn square(self, pd: Self, duty: Self) -> Self;

    fn u8(self) -> u8;
}

impl Float for f32 {
    fn step(self, threshold: f32) -> f32 {
        if self < threshold {
            0.0
        } else {
            1.0
        }
    }

    fn fmod(self, v: f32) -> f32 {
        self.rem_euclid(v)
    }

    fn lerp<R: Into<Range>>(self, onto: R) -> f32 {
        let (i, j) = onto.into().bounds();
        i + self * (j - i)
    }
    fn ilerp<R: Into<Range>>(self, from: R) -> f32 {
        let (i, j) = from.into().bounds();
        (self - i) / (j - i)
    }
    fn phase(self, pd: f32, fr: f32) -> f32 {
        (self + (fr * pd)).rem_euclid(pd)
    }

    fn line(self, slope: f32, intercept: f32) -> f32 {
        (self * slope) + intercept
    }
    fn cover(self, amt: f32) -> f32 {
        self.line(amt, (1.0 - amt) / 2.0)
    }

    fn fsin(self, pd: f32) -> f32 {
        let t = (2.0 * PI * self) / pd + (PI / 2.0);
        0.5 * t.sin() + 0.5
    }
    fn fcos(self, pd: f32) -> f32 {
        self.phase(pd, 0.5).fsin(pd)
    }
    fn ramp(self, pd: f32) -> f32 {
        self.fmod(pd) / pd
    }
    fn tri(self, pd: f32) -> f32 {
        let ramp = (2.0 * self - pd).fmod(2.0 * pd);
        (ramp - pd).abs() / pd
    }
    fn square(self, pd: f32, duty: f32) -> f32 {
        self.fmod(pd).step(pd * duty)
    }

    fn u8(self) -> u8 {
        (self * 255.0).floor() as u8
    }
}

pub fn u8(fr: f32) -> u8 {
    (fr * 255.0).floor() as u8
}
