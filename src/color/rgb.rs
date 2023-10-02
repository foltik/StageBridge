#[derive(Default, Clone, Copy, Debug)]
pub struct Rgb(pub f64, pub f64, pub f64);

impl Rgb {
    pub fn black() -> Self { Self(0.0, 0.0, 0.0) }
    pub fn off() -> Self { Self::black() }

    pub fn r(&self) -> f64 { self.0 }
    pub fn g(&self) -> f64 { self.1 }
    pub fn b(&self) -> f64 { self.2 }
}


#[derive(Default,Clone, Copy, Debug)]
pub struct Rgbw(pub f64, pub f64, pub f64, pub f64);

impl Rgbw {
    pub fn white() -> Self { Self(0.0, 0.0, 0.0, 1.0) }
    pub fn black() -> Self { Self(0.0, 0.0, 0.0, 0.0) }
    pub fn off() -> Self { Self::black() }

    pub fn r(&self) -> f64 { self.0 }
    pub fn g(&self) -> f64 { self.1 }
    pub fn b(&self) -> f64 { self.2 }
    pub fn w(&self) -> f64 { self.3 }
}
