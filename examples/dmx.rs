#![feature(async_closure)]

use std::{thread, sync::Arc};
use parking_lot::Mutex;

use anyhow::Result;

use stagebridge::{midi::Midi, e131::{E131, E131_PORT}};
use stagebridge::midi::device::launch_control_xl::types::{Color, Brightness};
use stagebridge::midi::device::launch_control_xl::{LaunchControlXL, Input, Output};

use tokio::{task::spawn, sync::mpsc};
use stagebridge::util::future::Broadcast;

fn byte(f: f32) -> u8 {
    (f.clamp(0.0, 1.0) * 255.0) as u8
}

pub trait Fixture {
    fn write(&self, channel: usize, data: &mut [u8]);
}

struct Head {
    pub mode: Mode,

    pub pitch: f32,
    pub yaw: f32,

    pub color: [f32; 4],
    pub brightness: f32,
    pub strobe: f32,

    pub ring: Ring,
}

enum Mode {
    Manual,
    ColorCycle,
    Auto,
}

impl Mode {
    pub fn byte(&self) -> u8 {
        match self {
            Mode::Manual => 0,
            Mode::ColorCycle => 159,
            Mode::Auto => 60,
        }
    }
}

pub enum Ring {
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

impl Ring {
    pub fn byte(&self) -> u8 {
        match self {
            Ring::Off => 0,

            Ring::Red => 4,
            Ring::Green => 22,
            Ring::Blue => 36,
            Ring::Yellow => 56,
            Ring::Purple => 74,
            Ring::Teal => 84,
            Ring::White => 104,

            Ring::RedYellow => 116,
            Ring::RedPurple => 128,
            Ring::RedWhite => 140,

            Ring::GreenYellow => 156,
            Ring::GreenBlue => 176,
            Ring::GreenWhite => 192,

            Ring::BluePurple => 206,
            Ring::BlueTeal => 216,
            Ring::BlueWhite => 242,

            Ring::Cycle => 248,
            Ring::Raw(i) => *i,
        }
    }
}

impl std::default::Default for Head {
    fn default() -> Self {
        Self {
            mode: Mode::Manual,

            pitch: 0.0,
            yaw: 0.0,

            color: [0.0; 4],
            brightness: 1.0,
            strobe: 0.0,

            ring: Ring::Off,
        }
    }
}

impl Fixture for Head {
    fn write(&self, channel: usize, data: &mut [u8]) {
        data[channel + 0] = byte(self.yaw);
        data[channel + 2] = byte(self.pitch);

        data[channel + 5] = byte(self.brightness);

        data[channel + 7] = byte(self.color[0]);
        data[channel + 8] = byte(self.color[1]);
        data[channel + 9] = byte(self.color[2]);
        data[channel + 10] = byte(self.color[3]);

        // data[channel + 11] = byte(self.color[0]);
        // data[channel + 12] = byte(self.color[1]);

        data[channel + 14] = self.ring.byte();
    }
}

struct State {
    heads: Head,
}

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let data = Arc::new(Mutex::new(vec![0; 512]));
    let state = Arc::new(Mutex::new(State {
        heads: Head::default(),
    }));

    let data_e131 = Arc::clone(&data);
    let state_e131 = Arc::clone(&state);
    thread::spawn(move || {
        let mut e131 = E131::new("10.10.2.1".parse().unwrap(), E131_PORT, 1).unwrap();

        loop {
            let mut data = data_e131.lock();
            let state = state_e131.lock();
            state.heads.write(81, &mut data);
            state.heads.write(97, &mut data);
            state.heads.write(112, &mut data);
            state.heads.write(127, &mut data);

            // log::debug!("{:?}", &data[81..81+15]);

            e131.send(&data);
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    });

    let rate = Arc::new(Mutex::new(60.0)); // hz
    let rate_inner = Arc::clone(&rate);
    let state_inner = Arc::clone(&state);
    thread::spawn(move || {
        loop {
            {
                let mut heads = &mut state_inner.lock().heads;
                heads.brightness = 1.0 - heads.brightness;
            }
            let delay = std::time::Duration::from_secs_f32(60.0 / *rate_inner.lock());
            std::thread::sleep(delay);
        }
    });

    match Midi::<LaunchControlXL>::open("Launch Control XL:Launch Control XL") {
        Ok(device) => {
            spawn(async move {
                let mut rx = device.subscribe();

                loop {
                    use tokio::sync::broadcast::error::RecvError;
                    match rx.recv().await {
                        Ok(input) => {
                            let mut state = state.lock();
                            log::debug!("{:?}", input);

                            match input {
                                Input::Slider(0, fr) => state.heads.yaw = fr,
                                Input::Slider(1, fr) => state.heads.pitch = fr,
                                Input::Slider(2, fr) => state.heads.color[0] = fr,
                                Input::Slider(3, fr) => state.heads.color[1] = fr,
                                Input::Slider(4, fr) => state.heads.color[2] = fr,
                                Input::Slider(5, fr) => state.heads.color[3] = fr,
                                // Input::Slider(6, fr) => state.heads.ring = fr,


                                Input::Slider(7, fr) => *rate.lock() = fr * 10000.0,

                                Input::Control(0, true) => state.heads.yaw = 0.0,
                                Input::Control(1, true) => state.heads.yaw = 1.0,
                                _ => {}
                            }
                        },
                        Err(RecvError::Lagged(n)) => log::warn!("lagged by {}", n),
                        Err(RecvError::Closed) => {
                            log::info!("closed");
                            return
                        },
                    }
                }
            });
        },
        Err(err) => {
            log::error!("Failed to connect to device: {:?}", err);
        }
    };

    std::thread::park();
    Ok(())
}
