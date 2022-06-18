use std::time::Duration;
use tokio::time::sleep;

use tokio::task::spawn;
use stagebridge::util::future::spawn_cancel;
use tokio_util::sync::CancellationToken;

use stagebridge::midi::device::launchpad_x::{*, types::*};
use stagebridge::util::ease::u8;

use crate::{Context, State};
use crate::lights::Color;

pub fn strobe(ctx: &'static Context, color: PaletteColor) -> CancellationToken {
    let lights = &ctx.lights;
    spawn_cancel(async move {
        let State { 
            bpm,
            div: (num, denom),
            duty,
            alpha,
            .. 
        } = ctx.state.get().await;

        let t = Duration::from_secs_f32((60.0 / (bpm / 4.0)) * (num as f32 / denom as f32)).mul_f32(duty);

        // let div = div.0 as f32 / div.1 as f32;
        // let q = ctx.beats.quantum();

        let update_pad = async move |color| {
            if let Some(pad) = ctx.pad.as_ref() {
                if !ctx.state.paused() {
                    for i in 0..64 {
                        pad.send(Output::Light(Index(i).into(), color)).await;
                    }
                }
            }
        };

        let mut on = false;

        let mut rx = ctx.beats.subscribe_div(num, denom);
        while let Some(_) = rx.recv().await {
            update_pad(color).await;
            lights.set_all(Color::from(color).a(u8(alpha))).await;

            spawn(async move {
                sleep(t).await;
                update_pad(PaletteColor::Off).await;
                lights.set_all(Color::OFF).await;

            });

            on = !on;
        }
    })
}

pub fn strobe_accent(ctx: &'static Context, color: PaletteColor) -> CancellationToken {
    let lights = &ctx.lights;
    spawn_cancel(async move {
        let State { 
            bpm,
            div: (num, denom),
            duty,
            alpha,
            .. 
        } = ctx.state.get().await;

        let t = Duration::from_secs_f32((60.0 / (bpm / 4.0)) * (num as f32 / denom as f32)).mul_f32(duty);

        // let div = div.0 as f32 / div.1 as f32;
        // let q = ctx.beats.quantum();

        let update_pad = async move |color| {
            if let Some(pad) = ctx.pad.as_ref() {
                if !ctx.state.paused() {
                    for i in 0..64 {
                        pad.send(Output::Light(Index(i).into(), color)).await;
                    }
                }
            }
        };

        let mut on = false;

        let mut rx = ctx.beats.subscribe_div(num, denom);
        while let Some(_) = rx.recv().await {
            update_pad(color).await;
            lights.set_all(Color::from(color).a(u8(alpha))).await;

            spawn(async move {
                sleep(t).await;
                update_pad(PaletteColor::Off).await;
                lights.set_all(Color::OFF).await;

            });

            on = !on;
        }
    })
}