use std::fmt::Debug;

pub trait Device: Send + 'static {
    type Input: Send + Debug;
    type Output: Send + Debug;

    fn process_input(&mut self, data: &[u8]) -> Option<Self::Input>;
    fn process_output(&mut self, output: Self::Output) -> Vec<u8>;
}

pub mod worlde_easycontrol9;
pub mod launchpad_x;
pub mod launch_control_xl;
