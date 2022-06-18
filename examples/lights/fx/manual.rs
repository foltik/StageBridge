use stagebridge::util::future::{Broadcast, spawn_cancel};
use tokio_util::sync::CancellationToken;

use stagebridge::midi::device::launchpad_x::{*, types::*};
use stagebridge::util::ease::{project, u8};

use crate::{Context, State};
use crate::lights::Color;

pub fn manual(ctx: &'static Context, color: PaletteColor) -> CancellationToken {
    let lights = &ctx.lights;
    let pad = ctx.pad.as_ref().unwrap();
    spawn_cancel(async move {
        let State { 
            alpha, 
            min,
            max,
            ..
        } = ctx.state.get().await;

        let update_pad = async move |fr, color| {
            if let Some(pad) = ctx.pad.as_ref() {
                if !ctx.state.paused() {
                    pad.send(Output::Brightness(fr)).await;
                    for i in 0..64 {
                        pad.send(Output::Light(Index(i).into(), color)).await;
                    }
                }
            }
        };

        let mut token = CancellationToken::new();

        let mut rx = pad.subscribe();
        loop {
            use tokio::sync::broadcast::error::RecvError;
            match rx.recv().await {
                Ok(input) => match input {
                    Input::Press(i, fr) => {
                        let Coord(x, y) = Pos::from(i).into();
                        if y == 6 || y == 7 && x <= 7 {
                            let q = ctx.beats.quantum();
                            let n = match x {
                                0..=1 => q / 16,
                                2..=3 => q / 8,
                                4..=5 => q / 4,
                                6     => q / 2,
                                7     => q,
                                _     => q / 16,
                            };

                            let mut i = 0;

                            token.cancel();
                            token = spawn_cancel(async move {
                                let mut rx = ctx.beats.subscribe();
                                while let Some(_) = rx.recv().await {
                                    if i <= n {
                                        log::debug!("{} -> {}", i, fr);
                                        let fr = 1.0 - (i as f32 / n as f32);
                                        let fr = project(min, max)(fr);

                                        lights.set_all(Color::from(color).a(u8(fr * alpha))).await;
                                        update_pad(fr, color).await;

                                        i += 1;
                                    } else {

                                        break;
                                    }
                                }
                                log::debug!("manual exit");
                            });
                        }
                    },
                    _ => {},
                },
                Err(RecvError::Lagged(n)) => log::warn!("manual recv lagged by {}", n),
                Err(RecvError::Closed) => log::error!("manual recv closed"),
            }
        }
    })
}