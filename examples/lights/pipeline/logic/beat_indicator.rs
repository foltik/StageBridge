use std::time::Duration;
use tokio::time::sleep;
use tokio::task::spawn;

use stagebridge::midi::device::launchpad_x::{*, types::*};

use crate::Context;

pub fn launch(ctx: &'static Context) {
    let pad = ctx.pad.as_ref().unwrap();

    spawn(async move {
        let mut rx = ctx.beats.subscribe_div(1, 4);
        while let Some(_) = rx.recv().await {
            spawn(async move {
                pad.send(Output::Light(Coord(8, 8).into(), PaletteColor::Index(53))).await;
                // ctx.lights.set_all(255, 255, 0, 255, 0).await;
                sleep(Duration::from_millis(25)).await;
                pad.send(Output::Off(Coord(8, 8).into())).await;
                // ctx.lights.set_all(0, 255, 0, 255, 0).await;
            });
        }
    });
}