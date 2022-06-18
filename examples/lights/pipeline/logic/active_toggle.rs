use std::time::Duration;
use tokio::time::sleep;

use tokio_util::sync::CancellationToken;
use stagebridge::util::future::{spawn_cancel, spawn_cancel_from};
use stagebridge::util::future::Broadcast;

use stagebridge::midi::device::launchpad_x::*;

use crate::{Command, Context, State};


pub fn launch(ctx: &'static Context) -> CancellationToken {
    let pad = ctx.pad.as_ref().unwrap();

    let token = CancellationToken::new();

    spawn_cancel_from(token.clone(), async move {
        let mut rx = pad.subscribe();

        let mut hold_token = CancellationToken::new();

        loop {

            use tokio::sync::broadcast::error::RecvError;
            match rx.recv().await {
                Ok(i) => match i {
                    Input::Capture(true) => {
                        let cmd = match ctx.state.get().await {
                            State { active, .. } => Command::Active(!active),
                        };

                        hold_token = spawn_cancel(async move {
                            sleep(Duration::from_secs(1)).await;
                            ctx.state.send(cmd).await;
                        });
                    }
                    Input::Capture(false) => {
                        hold_token.cancel();
                    }
                    _ => {}
                },
                Err(RecvError::Lagged(n)) => log::warn!("active toggle rx lagged by {}", n),
                Err(RecvError::Closed) => break,
            }
        }
    });

    spawn_cancel_from(token.clone(), async move {
        let mut rx = ctx.state.subscribe();
        loop {
            use tokio::sync::broadcast::error::RecvError;
            match rx.recv().await {
                Ok(State { active, .. }) => {
                    let pad = ctx.pad.as_ref().unwrap();
                    use stagebridge::midi::device::launchpad_x::{types::*, *};

                    if active {
                        pad.send(Output::Light(Coord(7, 8).into(), PaletteColor::Index(55)))
                            .await;
                    } else {
                        pad.send(Output::Light(Coord(7, 8).into(), PaletteColor::Index(1)))
                            .await;
                    }
                }
                Err(RecvError::Lagged(n)) => log::warn!("active toggle rx lagged by {}", n),
                Err(RecvError::Closed) => break,
            }
        }
    });

    token
}
