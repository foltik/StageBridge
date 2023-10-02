#[derive(Clone, Copy, Debug)]
pub struct Range {
    pub lo: f64,
    pub hi: f64,
}

impl Range {
    pub fn invert(self) -> Range {
        Self {
            lo: self.hi,
            hi: self.lo,
        }
    }

    pub fn bounds(self) -> (f64, f64) {
        (self.lo, self.hi)
    }

    pub fn order(self) -> Self {
        if self.lo < self.hi {
            self
        } else {
            self.invert()
        }
    }
}

macro_rules! impl_from {
    ($($ty:ident),*) => {
        $(
            impl From<std::ops::Range<$ty>> for Range {
                fn from(r: std::ops::Range<$ty>) -> Self {
                    Self {
                        lo: r.start as f64,
                        hi: r.end as f64,
                    }
                }
            }
        )*
    };
}

impl_from!(f64, i32);
