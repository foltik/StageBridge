use tokio_util::sync::CancellationToken;
use stagebridge::util::future::spawn_cancel_from;
use stagebridge::util::future::Broadcast;

use stagebridge::midi::device::launchpad_x::{*, types::*};

use crate::{Context, State, Command};


pub fn launch(ctx: &'static Context) -> CancellationToken {
    let pad = ctx.pad.as_ref().unwrap();

    let token = CancellationToken::new();

    spawn_cancel_from(token.clone(), async move {
        let mut rx = pad.subscribe();
        loop {
            use tokio::sync::broadcast::error::RecvError;
            match rx.recv().await {
                Ok(i) => match i {
                    Input::Stop(b) => { 
                        ctx.state.send(Command::PauseEnable(b)).await;
                    },
                    _ => {}
                },
                Err(RecvError::Lagged(n)) => log::warn!("pause enable rx lagged by {}", n),
                Err(RecvError::Closed) => break,
            }
        }
    });

    spawn_cancel_from(token.clone(), async move {
        let mut rx = ctx.state.subscribe();
        loop {
            use tokio::sync::broadcast::error::RecvError;
            match rx.recv().await {
                Ok(State { pause_enable, .. }) => {
                    if pause_enable {
                        pad.send(Output::Light(Coord(8, 3).into(), PaletteColor::Index(53))).await;
                    } else {
                        pad.send(Output::Light(Coord(8, 3).into(), PaletteColor::Index(1))).await;
                    }
                },
                Err(RecvError::Lagged(n)) => log::warn!("pause enable rx lagged by {}", n),
                Err(RecvError::Closed) => break,
            }
        }
    });

    token
}