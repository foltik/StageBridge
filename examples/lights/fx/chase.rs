// use tokio::task::spawn;
use stagebridge::util::future::spawn_cancel;
use tokio_util::sync::CancellationToken;

use stagebridge::midi::device::launchpad_x::{*, types::*};
use stagebridge::util::ease::{dt, u8, map, saw_up};

use crate::{Context, State};
use crate::lights::Color;

pub fn chase(ctx: &'static Context, primary: PaletteColor, secondary: PaletteColor) -> CancellationToken {
    let lights = &ctx.lights;
    spawn_cancel(async move {
        let State { 
            div: (num, denom), 
            ofs_light,
            duty,
            alpha,
            .. 
        } = ctx.state.get().await;

        let q = ctx.beats.quantum();
        let div = (q * num) / denom;

        let update_pad = async move |on| {
            if let Some(pad) = ctx.pad.as_ref() {
                if !ctx.state.paused() {
                    for i in 0..64 {
                        if on {
                            pad.send(Output::Light(Index(i).into(), primary)).await;
                        } else {
                            pad.send(Output::Off(Index(i).into())).await;
                        }
                    }
                }
            }
        };

        let mut rx = ctx.beats.subscribe_div(1, q);
        while let Some(b) = rx.recv().await {
            let t = b.rem_euclid(div) as f32 / div as f32;
            let tt_light = dt(t, ofs_light);
            // let tt_pad = dt(t, ofs_pad);

            let fr_light = saw_up(1.0)(tt_light);
            // let fr_pad = saw_up(1.0)(tt_pad);

            let i = map(0.0, 1.0, 0.0, 10.0)(fr_light).floor() as usize;
            let j = map(0.0, 1.0, 0.0, 10.0)(dt(fr_light, duty)).floor() as usize;
            
            // log::debug!("{}: {} -> {}", fr_light, i0, j0);

            let primary = Color::from(primary).a(u8(alpha));
            let secondary = Color::from(secondary).a(u8(alpha));

            update_pad(i >= 9).await;


            pub fn set(buffer: &mut [u8], i: usize, c: Color) {
                buffer[8*i + 4] = c.a;
                buffer[8*i + 5] = c.r;
                buffer[8*i + 6] = c.g;
                buffer[8*i + 7] = c.b;
                buffer[8*i + 8] = c.w;
            }

            if j < i {
                let mut buffer = vec![0; 81];
                for n in i..10 {
                    set(&mut buffer, n, primary);
                }
                for n in 0..=j {
                    set(&mut buffer, n, primary);
                }
                for n in (j + 1)..i {
                    set(&mut buffer, n, secondary);
                }
                lights.send(buffer).await;
            } else {
                let mut buffer = vec![0; 81];
                for n in 0..j {
                    set(&mut buffer, n, secondary);
                }
                for n in i..=j {
                    set(&mut buffer, n, primary);
                }
                for n in (j + 1)..10 {
                    set(&mut buffer, n, secondary);
                }
                lights.send(buffer).await;
            }
        }
    })
}