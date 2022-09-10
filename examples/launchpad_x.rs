#![feature(async_closure)]

use anyhow::Result;

use stagebridge::midi::Midi;
use stagebridge::midi::device::launchpad_x::{LaunchpadX, Output};
use stagebridge::midi::device::launchpad_x::types::{Pressure, PressureCurve, Mode};

use tokio::task::spawn;
use stagebridge::util::future::Broadcast;

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    match Midi::<LaunchpadX>::open("Launchpad X:Launchpad X LPX MIDI") {
        Ok(device) => {
            log::info!("Listening...");
            device.send(Output::Mode(Mode::Programmer)).await;
            device.send(Output::Pressure(Pressure::Off, PressureCurve::Medium)).await;
            device.send(Output::Clear).await;

            spawn(async move {
                let mut rx = device.subscribe();
                loop {
                    use tokio::sync::broadcast::error::RecvError;
                    match rx.recv().await {
                        Ok(input) => {
                            log::info!("{:?}", input);

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
