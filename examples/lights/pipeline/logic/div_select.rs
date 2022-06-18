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
                Ok(i) => {
                    if let Some(div) = match i {
                        Input::Custom(true)  => Some((2, 1)),
                        Input::Note(true)    => Some((1, 1)),
                        Input::Session(true) => Some((1, 2)),
                        Input::Right(true)   => Some((1, 4)),
                        Input::Left(true)    => Some((1, 8)),
                        Input::Down(true)    => Some((1, 16)),
                        Input::Up(true)      => Some((1, 32)),
                        _ => None
                    } {
                        ctx.state.send(Command::Div(div)).await;
                    }
                }
                Err(RecvError::Lagged(n)) => log::warn!("div select rx lagged by {}", n),
                Err(RecvError::Closed) => break,
            }
        }
    });

    spawn_cancel_from(token.clone(), async move {
        let mut rx = ctx.state.subscribe_mode();
        loop {
            use tokio::sync::broadcast::error::RecvError;
            match rx.recv().await {
                Ok(State { div, .. }) => {
                    for x in 0..=6 {
                        pad.send(Output::Light(Coord(x, 8).into(), PaletteColor::Index(1))).await;
                    }

                    if let Some(x) = match div {
                        (2, 1)  => Some(6),
                        (1, 1)  => Some(5),
                        (1, 2)  => Some(4),
                        (1, 4)  => Some(3),
                        (1, 8)  => Some(2),
                        (1, 16) => Some(1),
                        (1, 32) => Some(0),
                        _ => None,
                    } {
                        pad.send(Output::Light(Coord(x, 8).into(), PaletteColor::Index(55))).await;
                    }
                } 
                Err(RecvError::Lagged(n)) => log::warn!("div select rx lagged by {}", n),
                Err(RecvError::Closed) => break,
            }
        }
    });

    token
}