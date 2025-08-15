use super::MidiDevice;
use crate::midi::Midi;

pub mod types;
use types::*;

#[derive(Debug)]
pub struct LaunchControlXL {
    /// Cache of last requested (Color, Brightness) per hardware LED index.
    cache: [Option<(Color, Brightness)>; 0x30],
}

impl Default for LaunchControlXL {
    fn default() -> Self {
        Self { cache: [None; 0x30] }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Input {
    Slider(u8, f64),

    SendA(u8, f64),
    SendB(u8, f64),
    Pan(u8, f64),

    Mode(Mode),
    TrackSelect(bool, bool),
    SendSelect(bool, bool),

    Device(bool),
    Mute(bool),
    Solo(bool),
    Record(bool),

    Focus(u8, bool),
    Control(u8, bool),

    Unknown,
}

#[derive(Copy, Clone, Debug)]
pub enum Output {
    SendA(u8, Color, Brightness),
    SendB(u8, Color, Brightness),
    Pan(u8, Color, Brightness),

    Focus(u8, Color, Brightness),
    Control(u8, Color, Brightness),

    TrackSelect(bool, Color, Brightness),
    SendSelect(bool, Color, Brightness),

    Device(Color, Brightness),
    Mute(Color, Brightness),
    Solo(Color, Brightness),
    Record(Color, Brightness),
}

fn float(v: u8) -> f64 {
    (v as f64) / 127.0
}

fn float_diverging(v: u8) -> f64 {
    if v >= 0x40 {
        ((v - 0x40) as f64) / 63.0
    } else {
        -1.0 + ((v as f64) / 64.0)
    }
}

fn color_mask(c: Color, b: Brightness) -> u8 {
    let color_mask = match c {
        Color::Red => b.byte(),
        Color::Amber => b.byte() | (b.byte() << 4),
        Color::Green => b.byte() << 4,
    };

    color_mask | 0b1100
}

fn light_sysex(idx: u8, color: Color, brightness: Brightness) -> Vec<u8> {
    vec![
        0xf0,
        0x00,
        0x20,
        0x29,
        0x02,
        0x11,
        0x78,
        0x00,
        idx,
        color_mask(color, brightness),
        0xf7,
    ]
}

impl LaunchControlXL {
    /// Map a logical Output to the hardware LED index and desired (Color, Brightness).
    #[inline]
    fn map_output(o: &Output) -> (u8, Color, Brightness) {
        match *o {
            Output::SendA(idx, col, b) => (idx, col, b),
            Output::SendB(idx, col, b) => (idx + 0x08, col, b),
            Output::Pan(idx, col, b) => (idx + 0x10, col, b),

            Output::Focus(idx, col, b) => (idx + 0x18, col, b),
            Output::Control(idx, col, b) => (idx + 0x20, col, b),

            Output::SendSelect(i, col, b) => (if i { 0x2c } else { 0x2d }, col, b),
            Output::TrackSelect(i, col, b) => (if i { 0x2e } else { 0x2f }, col, b),

            Output::Device(col, b) => (0x28, col, b),
            Output::Mute(col, b) => (0x29, col, b),
            Output::Solo(col, b) => (0x2a, col, b),
            Output::Record(col, b) => (0x2b, col, b),
        }
    }
}

impl MidiDevice for LaunchControlXL {
    type Input = Input;
    type Output = Output;

    fn init(ctrl: &mut Midi<Self>) {
        use types::*;
        for i in 0..8 {
            ctrl.send(Output::SendA(i, Color::Red, Brightness::Off));
            ctrl.send(Output::SendB(i, Color::Red, Brightness::Off));
            ctrl.send(Output::Pan(i, Color::Red, Brightness::Off));
            ctrl.send(Output::Focus(i, Color::Red, Brightness::Off));
            ctrl.send(Output::Control(i, Color::Red, Brightness::Off));
        }
        ctrl.send(Output::SendSelect(true, Color::Red, Brightness::Off));
        ctrl.send(Output::SendSelect(false, Color::Red, Brightness::Off));
        ctrl.send(Output::TrackSelect(true, Color::Red, Brightness::Off));
        ctrl.send(Output::TrackSelect(false, Color::Red, Brightness::Off));
        ctrl.send(Output::Device(Color::Red, Brightness::Off));
        ctrl.send(Output::Mute(Color::Red, Brightness::Off));
        ctrl.send(Output::Solo(Color::Red, Brightness::Off));
        ctrl.send(Output::Record(Color::Red, Brightness::Off));
    }

    fn process_input(&mut self, raw: &[u8]) -> Option<Input> {
        Some(match raw[0] & 0xf0 {
            0xf0 => Input::Mode(match raw[7] {
                0x0 => Mode::User,
                0x8 => Mode::Factory,
                _ => return None,
            }),
            0x90 => match raw[1] {
                0x29..=0x2c => Input::Focus(raw[1] - 0x29, true),
                0x39..=0x3c => Input::Focus(4 + raw[1] - 0x39, true),
                0x49..=0x4c => Input::Control(raw[1] - 0x49, true),
                0x59..=0x5c => Input::Control(4 + raw[1] - 0x59, true),
                0x69 => Input::Device(true),
                0x6a => Input::Mute(true),
                0x6b => Input::Solo(true),
                0x6c => Input::Record(true),
                _ => return None,
            },
            0x80 => match raw[1] {
                0x29..=0x2c => Input::Focus(raw[1] - 0x29, false),
                0x39..=0x3c => Input::Focus(4 + raw[1] - 0x39, false),
                0x49..=0x4c => Input::Control(raw[1] - 0x49, false),
                0x59..=0x5c => Input::Control(4 + raw[1] - 0x59, false),
                0x69 => Input::Device(false),
                0x6a => Input::Mute(false),
                0x6b => Input::Solo(false),
                0x6c => Input::Record(false),
                _ => return None,
            },
            0xb0 => match raw[1] {
                0x0d..=0x14 => Input::SendA(raw[1] - 0x0d, float_diverging(raw[2])),
                0x1d..=0x24 => Input::SendB(raw[1] - 0x1d, float_diverging(raw[2])),
                0x31..=0x38 => Input::Pan(raw[1] - 0x31, float_diverging(raw[2])),
                0x4d..=0x54 => Input::Slider(raw[1] - 0x4d, float(raw[2])),
                0x68..=0x69 => Input::SendSelect(raw[1] == 0x69, raw[2] == 0x7f),
                0x6a..=0x6b => Input::TrackSelect(raw[1] == 0x6b, raw[2] == 0x7f),
                _ => return None,
            },
            _ => return None,
        })
    }

    fn process_output(&mut self, output: Output) -> Vec<u8> {
        let (idx, col, b) = LaunchControlXL::map_output(&output);
        let slot = idx as usize;

        let new_mask = color_mask(col, b);
        let changed = match self.cache[slot] {
            Some((old_c, old_b)) => color_mask(old_c, old_b) != new_mask,
            None => true,
        };

        if changed {
            // Update cache and emit sysex.
            self.cache[slot] = Some((col, b));
            light_sysex(idx, col, b)
        } else {
            // No change -> do nothing.
            Vec::new()
        }
    }
}
