use tokio_util::sync::CancellationToken;
use stagebridge::util::future::spawn_cancel_from;
use stagebridge::util::future::Broadcast;

use stagebridge::midi::device::launchpad_x::{*, types::*};

use crate::state::Palette;
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
                    Input::Volume(true) => { ctx.state.send(Command::Palette(Palette::Main)).await; },
                    Input::Pan(true) => { ctx.state.send(Command::Palette(Palette::Foo)).await; },
                    Input::A(true) => { ctx.state.send(Command::Palette(Palette::Bar)).await; },
                    Input::B(true) => { ctx.state.send(Command::Palette(Palette::Color)).await; },
                    _ => {}
                },
                Err(RecvError::Lagged(n)) => log::warn!("pallete switcher rx lagged by {}", n),
                Err(RecvError::Closed) => break,
            }
        }
    });

    spawn_cancel_from(token.clone(), async move {
        let mut rx = ctx.state.subscribe();
        loop {
            use tokio::sync::broadcast::error::RecvError;
            match rx.recv().await {
                Ok(State { palette, .. }) => {
                    for y in 4..=7 {
                        pad.send(Output::Light(Coord(8, y).into(), PaletteColor::Index(1))).await;
                    }

                    let y = match palette {
                        Palette::Main => 7,
                        Palette::Foo => 6,
                        Palette::Bar => 5,
                        Palette::Color => 4,
                    };
                    pad.send(Output::Light(Coord(8, y).into(), PaletteColor::Index(55))).await;
                },
                Err(RecvError::Lagged(n)) => log::warn!("pallete switcher rx lagged by {}", n),
                Err(RecvError::Closed) => break,
            }
        }
    });

    token
}