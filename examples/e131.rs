use anyhow::Result;
use std::time::Duration;

use stagebridge::color::Rgbw;
use stagebridge::dmx::device::par_rgbw_12x3w::Par;
use stagebridge::dmx::Device;
use stagebridge::e131::E131;

fn main() -> Result<()> {
    let pars = [Par { color: Rgbw::WHITE, alpha: 1.0 }; 10];
    let dmx = {
        let mut channels = vec![0; 8 * pars.len() + 1];
        for (i, par) in pars.iter().enumerate() {
            par.encode(&mut channels[1 + 8 * i..]);
        }
        channels
    };

    let mut e131 = E131::new()?;
    let dest = "10.16.4.1".parse()?;
    loop {
        e131.send(&dest, &dmx);
        std::thread::sleep(Duration::from_millis(5));
    }
}
