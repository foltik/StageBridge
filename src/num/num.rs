use std::f64::consts::{PI, FRAC_PI_2};

use super::Range;

pub trait Float: Sized {
    /// 0 below the threshold and 1 above it
    fn step(self, threshold: Self) -> Self;
    /// false below the threshold and true above it
    fn bstep(self, threshold: Self) -> bool;

    fn rclamp<R: Into<Range>>(self, range: R) -> Self;

    /// Euclidean remainder (proper float mod)
    fn fmod(self, v: Self) -> Self;
    /// Mod and then divide
    fn mod_div(self, pd: f64) -> Self;
    /// Interpolate self in 0..1 onto another range
    fn lerp<R: Into<Range>>(self, onto: R) -> Self;
    /// Interpolate self in 0..1 onto another range as a u8
    fn lerp_byte<R: Into<Range>>(self, onto: R) -> u8;
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

    fn ssin(self, pd: Self) -> Self;
    fn scos(self, pd: Self) -> Self;
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
    fn negsquare(self, pd: Self, duty: Self) -> Self;
    /// Square wave, but booleans
    fn bsquare(self, pd: Self, duty: Self) -> bool;

    /// Convert 0..1 to 0..255u8
    fn byte(self) -> u8;
    /// Convert 0..1 to 0..127u8
    fn midi_byte(self) -> u8;
}

impl Float for f64 {
    fn step(self, threshold: f64) -> f64 {
        if self < threshold {
            0.0
        } else {
            1.0
        }
    }
    fn bstep(self, threshold: f64) -> bool {
        self.step(threshold) == 1.0
    }

    fn rclamp<R: Into<Range>>(self, range: R) -> f64 {
        let range = range.into();
        self.clamp(range.lo, range.hi)
    }

    fn fmod(self, v: f64) -> f64 {
        self.rem_euclid(v)
    }
    fn mod_div(self, v: f64) -> f64 {
        self.fmod(v) / v
    }

    fn lerp<R: Into<Range>>(self, onto: R) -> f64 {
        let (i, j) = onto.into().bounds();
        i + self.clamp(0.0, 1.0) * (j - i)
    }
    fn lerp_byte<R: Into<Range>>(self, onto: R) -> u8 {
        self.lerp(onto) as u8
    }
    fn ilerp<R: Into<Range>>(self, from: R) -> f64 {
        let (i, j) = from.into().bounds();
        (self - i) / (j - i)
    }
    fn phase(self, pd: f64, fr: f64) -> f64 {
        (self + (fr * pd)).rem_euclid(pd)
    }

    fn line(self, slope: f64, intercept: f64) -> f64 {
        (self * slope) + intercept
    }
    fn cover(self, amt: f64) -> f64 {
        self.line(amt, (1.0 - amt) / 2.0)
    }

    fn ssin(self, pd: f64) -> f64 {
        let t = (2.0 * PI * self) / pd;
        t.sin()
    }
    fn scos(self, pd: f64) -> f64 {
        let t = (2.0 * PI * self) / pd;
        t.cos()
    }
    fn fsin(self, pd: f64) -> f64 {
        let t = (2.0 * PI * self) / pd + (PI / 2.0);
        0.5 * t.sin() + 0.5
    }
    fn fcos(self, pd: f64) -> f64 {
        self.phase(pd, 0.5).fsin(pd)
    }
    fn ramp(self, pd: f64) -> f64 {
        self.fmod(pd) / pd
    }
    fn tri(self, pd: f64) -> f64 {
        let ramp = (2.0 * self - pd).fmod(2.0 * pd);
        (ramp - pd).abs() / pd
    }
    fn square(self, pd: f64, duty: f64) -> f64 {
        1.0 - self.fmod(pd).step(pd * duty)
    }
    fn negsquare(self, pd: f64, duty: f64) -> f64 {
        2.0 * (1.0 - self.fmod(pd).step(pd * duty)) - 1.0
    }
    fn bsquare(self, pd: f64, duty: f64) -> bool {
        self.square(pd, duty) == 1.0
    }

    fn byte(self) -> u8 {
        self.lerp_byte(0..255)
    }
    fn midi_byte(self) -> u8 {
        self.lerp_byte(0..127)
    }
}


pub trait Byte: Sized {
    /// Convert 0..255 to 0..1f
    fn float(self) -> f64;
    /// Convert 0..127 to 0..1f
    fn midi_float(self) -> f64;
}

impl Byte for u8 {
    fn float(self) -> f64 {
        (self as f64) / 255.0
    }

    fn midi_float(self) -> f64 {
        (self as f64) / 127.0
    }
}

pub trait Ease: Sized {
    fn ease_quad_in(self) -> Self;
    fn ease_quad_out(self) -> Self;
    fn ease_quad_inout(self) -> Self;
    fn ease_cubic_in(self) -> Self;
    fn ease_cubic_out(self) -> Self;
    fn ease_cubic_inout(self) -> Self;
    fn ease_quartic_in(self) -> Self;
    fn ease_quartic_out(self) -> Self;
    fn ease_quartic_inout(self) -> Self;
    fn ease_exp_in(self) -> Self;
    fn ease_exp_out(self) -> Self;
    fn ease_exp_inout(self) -> Self;
    fn ease_sin_in(self) -> Self;
    fn ease_sin_out(self) -> Self;
    fn ease_sin_inout(self) -> Self;
}

impl Ease for f64 {
    fn ease_quad_in(self) -> f64 {
        self * self
    }
    fn ease_quad_out(self) -> f64 {
        -(self * (self - 2.))
    }
    fn ease_quad_inout(self) -> f64 {
        if self < 0.5 { 2. * self * self }
        else { (-2. * self * self) + self.mul_add(4., -1.) }
    }

    fn ease_cubic_in(self) -> f64 {
        self * self * self
    }
    fn ease_cubic_out(self) -> f64 {
        let y = self - 1.;
        y * y * y + 1.
    }
    fn ease_cubic_inout(self) -> f64 {
        if self < 0.5 { 4. * self * self * self }
        else {
            let y = self.mul_add(2., -2.);
            (y * y * y).mul_add(0.5, 1.)
        }
    }
    fn ease_quartic_in(self) -> f64 {
        self * self * self * self
    }
    fn ease_quartic_out(self) -> f64 {
        let y = self - 1.;
        (y * y * y).mul_add(1. - self, 1.)
    }
    fn ease_quartic_inout(self) -> f64 {
        if self < 0.5 { 8. * self * self * self * self }
        else {
            let y = self - 1.;
            (y * y * y * y).mul_add(-8., 1.)
        }
    }
    fn ease_sin_in(self) -> f64 {
        let y = (self - 1.) * FRAC_PI_2;
        y.sin() + 1.
    }
    fn ease_sin_out(self) -> f64 {
        (self * FRAC_PI_2).sin()
    }
    fn ease_sin_inout(self) -> f64 {
        if self < 0.5 { 0.5 * (1. - (self * self).mul_add(-4., 1.).sqrt()) }
        else          { 0.5 * ((self.mul_add(-2., 3.) * self.mul_add(2., -1.)).sqrt() + 1.) }
    }
    fn ease_exp_in(self) -> f64 {
        if self == 0. { 0. }
        else          { (10. * (self - 1.)).exp2() }
    }
    fn ease_exp_out(self) -> f64 {
        if self == 1. { 1. }
        else          { 1. - (-10. * self).exp2() }
    }
    fn ease_exp_inout(self) -> f64 {
        if      self == 1. { 1. }
        else if self == 0. { 0. }
        else if self < 0.5 { self.mul_add(20., -10.).exp2() * 0.5 }
        else               { self.mul_add(-20., 10.).exp2().mul_add(-0.5, 1.) }
    }
}
