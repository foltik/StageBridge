use tokio::sync::mpsc;
use std::thread;

use stagebridge::e131::{E131, E131_PORT};

#[derive(Clone)]
pub struct Lights {
    tx: mpsc::Sender<Vec<u8>>,
}

impl Lights {
    const CH: usize = 81;
    const N: usize = 10;

    pub fn new() -> Self {
        let (tx, mut rx) = mpsc::channel::<Vec<u8>>(16);

        thread::spawn(move || {
            let mut e131 = E131::new("10.10.2.1".parse().unwrap(), E131_PORT, 1).unwrap();

            // TODO
            // let mut buffer = vec![0; Self::CH];

            loop {
                match rx.try_recv() {
                    Ok(data) => {
                        e131.send(&data);
                    },
                    Err(_) => {},
                }
            }
        });

        Self {
            tx
        }
    }

    pub async fn set(&self, i: usize, c: Color) {
        let mut buffer = vec![0; Self::CH];
        buffer[8*i + 4] = c.a;
        buffer[8*i + 5] = c.r;
        buffer[8*i + 6] = c.g;
        buffer[8*i + 7] = c.b;
        buffer[8*i + 8] = c.w;
        self.tx.send(buffer).await.unwrap();
    }

    pub async fn set_all(&self, c: Color) {
        let mut buffer = vec![0; Self::CH];
        for i in 0..Self::N {
            buffer[8*i + 4] = c.a;
            buffer[8*i + 5] = c.r;
            buffer[8*i + 6] = c.g;
            buffer[8*i + 7] = c.b;
            buffer[8*i + 8] = c.w;
        }
        self.tx.send(buffer).await.unwrap();
    }

    pub async fn set_fn<F>(&self, f: F) 
    where
        F: Fn(usize) -> Color
    {
        let mut buffer = vec![0; Self::CH];
        for i in 0..Self::N {
            let c = f(i);
            buffer[8*i + 4] = c.a;
            buffer[8*i + 5] = c.r;
            buffer[8*i + 6] = c.g;
            buffer[8*i + 7] = c.b;
            buffer[8*i + 8] = c.w;
        }
        self.tx.send(buffer).await.unwrap();
    }

    pub async fn send(&self, buffer: Vec<u8>) {
        self.tx.send(buffer).await.unwrap();
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Color { pub a: u8, pub r: u8, pub g: u8, pub b: u8, pub w: u8 }
impl Color {
    pub const OFF:     Self = Self::argbw(0, 0, 0, 0, 0);
    pub const WHITE:   Self = Self::w(0xff);
    pub const RED:     Self = Self::rgb(0xff, 0x00, 0x00);
    pub const ORANGE:  Self = Self::rgb(0xff, 0x40, 0x00);
    pub const YELLOW:  Self = Self::rgb(0xff, 0xff, 0x00);
    pub const PEA:     Self = Self::rgb(0x88, 0xff, 0x00);
    pub const LIME:    Self = Self::rgb(0x00, 0xff, 0x00);
    pub const MINT:    Self = Self::rgb(0x00, 0xff, 0x44);
    pub const CYAN:    Self = Self::rgb(0x00, 0xcc, 0xff);
    pub const BLUE:    Self = Self::rgb(0x00, 0x00, 0xff);
    pub const VIOLET:  Self = Self::rgb(0x88, 0x00, 0xff);
    pub const MAGENTA: Self = Self::rgb(0xff, 0x00, 0xff);
    pub const PINK:    Self = Self::rgb(0xff, 0x61, 0xcc);

    pub const fn argbw(a: u8, r: u8, g: u8, b: u8, w: u8) -> Self { Self { a, r, g, b, w } }
    pub const fn argb(a: u8, r: u8, g: u8, b: u8) -> Self { Self::argbw(a, r, g, b, 0) }
    pub const fn aw(a: u8, w: u8) -> Self { Self::argbw(a, 0, 0, 0, w) }
    pub const fn rgbw(r: u8, g: u8, b: u8, w: u8) -> Self { Self::argbw(255, r, g, b, w) }
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self { Self::argb(255, r, g, b) }
    pub const fn w(w: u8) -> Self { Self::aw(255, w) }

    pub fn hsv(h: f32, s: f32, v: f32) -> Self {
        use stagebridge::util::ease::{mix, u8};
        let r = v * mix(1.0, (((h + 1.0      ).fract() * 6.0 - 3.0).abs() - 1.0).clamp(0.0, 1.0), s);
        let g = v * mix(1.0, (((h + 0.6666666).fract() * 6.0 - 3.0).abs() - 1.0).clamp(0.0, 1.0), s);
        let b = v * mix(1.0, (((h + 0.3333333).fract() * 6.0 - 3.0).abs() - 1.0).clamp(0.0, 1.0), s);
        Self::rgb(u8(r), u8(g), u8(b))
    }
}

impl Color {
    pub fn a(self, a: u8) -> Self {
        Self { a, r: self.r, g: self.g, b: self.b, w: self.w }
    }
}

use stagebridge::midi::device::launchpad_x::types::PaletteColor;
impl From<PaletteColor> for Color {
    fn from(p: PaletteColor) -> Self {
        match p {
            PaletteColor::Index(_)   => Color::WHITE,
            PaletteColor::Off        => Color::OFF,
            PaletteColor::White      => Color::WHITE,
            PaletteColor::Red        => Color::RED,
            PaletteColor::Orange     => Color::ORANGE,
            PaletteColor::Yellow     => Color::YELLOW,
            PaletteColor::Pea        => Color::PEA,
            PaletteColor::Lime       => Color::LIME,
            PaletteColor::Mint       => Color::MINT,
            PaletteColor::Cyan       => Color::CYAN,
            PaletteColor::Blue       => Color::BLUE,
            PaletteColor::Violet     => Color::VIOLET,
            PaletteColor::Magenta    => Color::MAGENTA,
            PaletteColor::Pink       => Color::PINK,
        }
    }
}