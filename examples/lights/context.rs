use stagebridge::osc::Osc;
use stagebridge::midi::Midi;
use stagebridge::midi::device::launchpad_x::{LaunchpadX};
use stagebridge::midi::device::worlde_easycontrol9::{WorldeEasyControl9};

use super::beatgrid::BeatGrid;
use super::lights::Lights;
use super::state::StateHolder;

pub struct Context {
    pub osc: Osc,

    pub pad: Option<Midi<LaunchpadX>>,
    pub ctrl: Option<Midi<WorldeEasyControl9>>,

    pub lights: Lights,
    pub beats: BeatGrid,

    pub state: StateHolder,
}

impl Context {
    pub async fn new() -> &'static Self {
        let osc = Osc::new(7777).await;

        let pad = match Midi::<LaunchpadX>::open("Launchpad X:Launchpad X LPX MIDI") {
            Ok(pad) => {
                use stagebridge::midi::device::launchpad_x::{*, types::*};
                pad.send(Output::Mode(Mode::Programmer)).await;
                pad.send(Output::Pressure(Pressure::Off, PressureCurve::Medium)).await;
                pad.send(Output::Clear).await;
                Some(pad)
            },
            Err(e) => {
                log::warn!("Failed to open Launchpad: {:?}", e);
                None
            }
        };

        let ctrl = match Midi::<WorldeEasyControl9>::open("WORLDE easy control") {
            Ok(ctrl) => Some(ctrl),
            Err(e) => {
                log::warn!("Failed to open EasyControl: {:?}", e);
                None
            }
        };

        let lights = Lights::new();

        let beats = BeatGrid::new(128);
        beats.start(&osc);

        let state = StateHolder::spawn();

        Box::leak(Box::new(Self {
            osc,

            pad,
            ctrl,

            lights,
            beats,

            state,
        }))
    }
}