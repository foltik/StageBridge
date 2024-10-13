use std::fmt::Debug;

use super::Midi;

pub trait MidiDevice: Sized + Send + 'static {
    type Input: Send + Debug;
    type Output: Send + Debug;

    fn process_input(&mut self, data: &[u8]) -> Option<Self::Input>;
    fn process_output(&mut self, output: Self::Output) -> Vec<u8>;

    fn init(_midi: &mut Midi<Self>) {}
}

pub mod worlde_easycontrol9;
pub mod launchpad_x;
pub mod launch_control_xl;
