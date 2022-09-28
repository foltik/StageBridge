#[derive(Clone, Copy, Debug)]
pub struct Range {
    pub lo: f32,
    pub hi: f32,
}

impl Range {
    pub fn invert(self) -> Range {
        Self {
            lo: self.hi,
            hi: self.lo,
        }
    }

    pub fn bounds(self) -> (f32, f32) {
        (self.lo, self.hi)
    }
}

macro_rules! impl_from {
    ($ty:ident) => {
        impl From<std::ops::Range<$ty>> for Range {
            fn from(r: std::ops::Range<$ty>) -> Self {
                Self {
                    lo: r.start as f32,
                    hi: r.end as f32,
                }
            }
        }
    };
    ($ty:ident, $($rest:ident),*) => {
        impl_from!{$ty}
        impl_from!{$($rest),*}
    };
}

impl_from!{
    f32, f64, i32
}
