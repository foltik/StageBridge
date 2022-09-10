#![feature(async_closure)]

use anyhow::Result;

use stagebridge::midi::Midi;
use stagebridge::midi::device::launch_control_xl::types::{Color, Brightness, State};
use stagebridge::midi::device::launch_control_xl::{LaunchControlXL, Input, Output};

use tokio::task::spawn;
use stagebridge::util::future::Broadcast;

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    match Midi::<LaunchControlXL>::open("Launch Control XL:Launch Control XL") {
        Ok(device) => {
            log::info!("Listening...");
            // device.send(Output::Mode(Mode::Programmer)).await;
            // device.send(Output::Pressure(Pressure::Off, PressureCurve::Medium)).await;
            // device.send(Output::Clear).await;

            spawn(async move {
                let mut rx = device.subscribe();
                loop {
                    use tokio::sync::broadcast::error::RecvError;
                    match rx.recv().await {
                        Ok(input) => {
                            log::info!("{:?}", input);

                            match input {
                                // Input::Focus(idx, true) => {
                                // }
                                // Input::Control(idx, true) => {
                                // }
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
