use std::time::Duration;
use tokio::time::sleep;

// use tokio::task::spawn;
use stagebridge::util::future::spawn_cancel;
use tokio_util::sync::CancellationToken;

use stagebridge::midi::device::launchpad_x::{*, types::*};
use stagebridge::util::ease::u8;

use crate::{Context, State};
use crate::lights::Color;

pub fn solid(ctx: &'static Context, color: PaletteColor) -> CancellationToken {
    let lights = &ctx.lights;
    spawn_cancel(async move {
        let State { alpha, .. } = ctx.state.get().await;

        let update_pad = async move || {
            if let Some(pad) = ctx.pad.as_ref() {
                if !ctx.state.paused() {
                    for i in 0..64 {
                        pad.send(Output::Light(Index(i).into(), color)).await;
                    }
                }
            }
        };

        loop {
            lights.set_all(Color::from(color).a(u8(alpha))).await;
            update_pad().await;

            sleep(Duration::from_millis(50)).await;
        }
    })
}

pub fn solid_hue(ctx: &'static Context) -> CancellationToken {
    let lights = &ctx.lights;
    spawn_cancel(async move {
        let State { alpha, ..  } = ctx.state.get().await;

        let q = ctx.beats.quantum();

        let update_pad = async move |c: Color| {
            if let Some(pad) = ctx.pad.as_ref() {
                if !ctx.state.paused() {
                    for i in 0..64 {
                        let c = (c.r / 2, c.g / 2, c.b / 2);
                        pad.send(Output::Rgb(Index(i).into(), c)).await;
                    }
                }
            }
        };

        let mut rx = ctx.beats.subscribe();
        while let Some(b) = rx.recv().await {
            let t = b.rem_euclid(4 * q) as f32 / (4 * q) as f32;
            let color = Color::hsv(t, 1.0, 1.0).a(u8(alpha));

            lights.set_all(color).await;
            update_pad(color).await;
        }
    })
}

// pub fn solid_hue_rotate(ctx: &'static Context) -> CancellationToken {
//     let lights = &ctx.lights;
//     spawn_cancel(async move {
//         let State { alpha, ..  } = ctx.state.get().await;

//         let q = ctx.beats.quantum();

//         let update_pad = async move |c: Color| {
//             if let Some(pad) = ctx.pad.as_ref() {
//                 if !ctx.state.paused() {
//                     for i in 0..64 {
//                         let c = (c.r / 2, c.g / 2, c.b / 2);
//                         pad.send(Output::Rgb(Index(i).into(), c)).await;
//                     }
//                 }
//             }
//         };

//         let mut rx = ctx.beats.subscribe();
//         while let Some(b) = rx.recv().await {
//             let t = b.rem_euclid(4 * q) as f32 / (4 * q) as f32;
//             let color = Color::hsv(t, 1.0, 1.0).a(u8(alpha));

//             lights.set_all(color).await;
//             update_pad(color).await;
//         }
//     })
// }