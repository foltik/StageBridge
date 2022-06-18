// use tokio::task::spawn;
use stagebridge::util::future::spawn_cancel;
use tokio_util::sync::CancellationToken;

use stagebridge::midi::device::launchpad_x::{*, types::*};
use stagebridge::util::ease::{dt, project, tri, u8};

use crate::{Context, State};
use crate::lights::Color;

pub fn bounce(ctx: &'static Context, color: PaletteColor) -> CancellationToken {
    let lights = &ctx.lights;
    spawn_cancel(async move {
        let State { 
            div, 
            ofs_light, ofs_pad, 
            min, max,
            alpha,
            .. 
        } = ctx.state.get().await;

        let div = div.0 as f32 / div.1 as f32;
        let q = ctx.beats.quantum();

        let update_pad = async move |fr| {
            if let Some(pad) = ctx.pad.as_ref() {
                if !ctx.state.paused() {
                    pad.send(Output::Brightness(fr)).await;
                    for i in 0..64 {
                        pad.send(Output::Light(Index(i).into(), color)).await;
                    }
                }
            }
        };

        let mut rx = ctx.beats.subscribe_div(1, q);
        while let Some(b) = rx.recv().await {
            let t = b as f32 / q as f32;
            let tt_light = dt(t, -div / 2.0 + ofs_light);
            let tt_pad = dt(t, -div / 2.0 + ofs_pad);


            let fr_light = project(min, max)(tri(div)(tt_light));
            let fr_pad = project(min, max)(tri(div)(tt_pad));

            let color = Color::from(color).a(u8(fr_light * alpha));
            lights.set_all(color).await;
            update_pad(fr_pad).await;
        }
    })
}

pub fn bounce_hue(ctx: &'static Context) -> CancellationToken {
    let lights = &ctx.lights;
    spawn_cancel(async move {
        let State { 
            div: (num, denom), 
            ofs_light, ofs_pad, 
            min, max,
            alpha,
            .. 
        } = ctx.state.get().await;

        let q = ctx.beats.quantum();
        let div = (q * num) / denom;

        let update_pad = async move |fr, c: Color| {
            if let Some(pad) = ctx.pad.as_ref() {
                if !ctx.state.paused() {
                    pad.send(Output::Brightness(fr)).await;
                    for i in 0..64 {
                        let c = (c.r / 2, c.g / 2, c.b / 2);
                        pad.send(Output::Rgb(Index(i).into(), c)).await;
                    }
                }
            }
        };

        let mut rx = ctx.beats.subscribe();
        while let Some(b) = rx.recv().await {
            let t_bounce = b.rem_euclid(div) as f32 / div as f32;
            let t_hue = b.rem_euclid(4 * q) as f32 / (4 * q) as f32;

            let tt_light = dt(t_bounce, -0.5 + ofs_light);
            let tt_pad = dt(t_bounce, -0.5 + ofs_pad);

            let fr_light = project(min, max)(tri(1.0)(tt_light));
            let fr_pad = project(min, max)(tri(1.0)(tt_pad));

            let color = Color::hsv(t_hue, 1.0, 1.0);

            lights.set_all(color.a(u8(alpha * fr_light))).await;
            update_pad(fr_pad, color.a(u8(alpha))).await;
        }
    })
}