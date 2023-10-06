use crate::num::Float;

#[derive(Default, Clone, Copy, Debug)]
pub struct Rgb(pub f64, pub f64, pub f64);

impl Rgb {
    pub const BLACK:   Self = Self(0.0,   0.0,   0.0);
    pub const WHITE:   Self = Self(1.0,   1.0,   1.0);
    pub const RGB:     Self = Self(1.0,   1.0,   1.0);
    pub const RED:     Self = Self(1.0,   0.0,   0.0);
    pub const ORANGE:  Self = Self(1.0,   0.251, 0.0);
    pub const YELLOW:  Self = Self(1.0,   1.0,   0.0);
    pub const PEA:     Self = Self(0.533, 1.0,   0.0);
    pub const LIME:    Self = Self(0.0,   1.0,   0.0);
    pub const MINT:    Self = Self(0.0,   1.0,   0.267);
    pub const CYAN:    Self = Self(0.0,   0.8,   1.0);
    pub const BLUE:    Self = Self(0.0,   0.0,   1.0);
    pub const VIOLET:  Self = Self(0.533, 0.0,   1.0);
    pub const MAGENTA: Self = Self(1.0,   0.0,   1.0);
    pub const PINK:    Self = Self(1.0,   0.38,  0.8);

    pub fn a(&self, a: f64) -> Self {
        let Self(r, g, b) = self;
        Self(r*a, g*a, b*a)
    }

    pub fn hsv(h: f64, s: f64, v: f64) -> Self {
        let r = v * s.lerp(1.0..(((h + (3.0 / 3.0)).fract() * 6.0 - 3.0).abs() - 1.0).clamp(0.0, 1.0));
        let g = v * s.lerp(1.0..(((h + (2.0 / 3.0)).fract() * 6.0 - 3.0).abs() - 1.0).clamp(0.0, 1.0));
        let b = v * s.lerp(1.0..(((h + (1.0 / 3.0)).fract() * 6.0 - 3.0).abs() - 1.0).clamp(0.0, 1.0));

        Self(r, g, b)
    }
}


#[derive(Default, Clone, Copy, Debug)]
pub struct Rgbw(pub f64, pub f64, pub f64, pub f64);

impl Rgbw {
    pub const BLACK:   Self = Self(0.0,   0.0,   0.0,   0.0);
    pub const WHITE:   Self = Self(1.0,   1.0,   1.0,   1.0);
    pub const RGB:     Self = Self(1.0,   1.0,   1.0,   0.0);
    pub const RGBW:    Self = Self(1.0,   1.0,   1.0,   1.0);
    pub const RED:     Self = Self(1.0,   0.0,   0.0,   0.0);
    pub const ORANGE:  Self = Self(1.0,   0.251, 0.0,   0.0);
    pub const YELLOW:  Self = Self(1.0,   1.0,   0.0,   0.0);
    pub const PEA:     Self = Self(0.533, 1.0,   0.0,   0.0);
    pub const LIME:    Self = Self(0.0,   1.0,   0.0,   0.0);
    pub const MINT:    Self = Self(0.0,   1.0,   0.267, 0.0);
    pub const CYAN:    Self = Self(0.0,   0.8,   1.0,   0.0);
    pub const BLUE:    Self = Self(0.0,   0.0,   1.0,   0.0);
    pub const VIOLET:  Self = Self(0.533, 0.0,   1.0,   0.0);
    pub const MAGENTA: Self = Self(1.0,   0.0,   1.0,   0.0);
    pub const PINK:    Self = Self(1.0,   0.38,  0.8,   0.0);

    pub fn a(&self, a: f64) -> Self {
        let Self(r, g, b, w) = self;
        Self(r*a, g*a, b*a, w*a)
    }

    pub fn hsv(h: f64, s: f64, v: f64) -> Self {
        let r = v * s.lerp(1.0..(((h + (3.0 / 3.0)).fract() * 6.0 - 3.0).abs() - 1.0).clamp(0.0, 1.0));
        let g = v * s.lerp(1.0..(((h + (2.0 / 3.0)).fract() * 6.0 - 3.0).abs() - 1.0).clamp(0.0, 1.0));
        let b = v * s.lerp(1.0..(((h + (1.0 / 3.0)).fract() * 6.0 - 3.0).abs() - 1.0).clamp(0.0, 1.0));

        // subtract common white component
        let w = r.min(g).min(b);
        Self(r-w, g-w, b-w, w)
    }
}

impl From<Rgbw> for Rgb {
    fn from(Rgbw(mut r, mut g, mut b, w): Rgbw) -> Self {
        // add white
        r += w;
        g += w;
        b += w;

        // normalize
        let max = r.max(g).max(b);
        if max > 1.0 {
            r /= max;
            g /= max;
            b /= max;
        }

        Self(r, g, b)
    }
}

impl From<Rgb> for Rgbw {
    fn from(Rgb(r, g, b): Rgb) -> Self {
        Self(r, g, b, 0.0)
    }
}
