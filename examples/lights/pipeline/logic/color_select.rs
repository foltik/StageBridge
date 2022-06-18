use tokio_util::sync::CancellationToken;
use stagebridge::util::future::spawn_cancel_from;
use stagebridge::util::future::Broadcast;

use stagebridge::midi::device::launchpad_x::{*, types::*};

use crate::{Command, Context};

fn palette(i: u8) -> Option<PaletteColor> {
    match i {
        0  => Some(PaletteColor::Red),
        1  => Some(PaletteColor::Orange),
        2  => Some(PaletteColor::Yellow),
        3  => Some(PaletteColor::Pea),
        4  => Some(PaletteColor::Lime),
        5  => Some(PaletteColor::Mint),
        6  => Some(PaletteColor::Cyan),
        7  => Some(PaletteColor::Blue),
        8  => Some(PaletteColor::Violet),
        9  => Some(PaletteColor::Magenta),
        10 => Some(PaletteColor::Pink),
        11 => Some(PaletteColor::White),
        12 => Some(PaletteColor::Off),
        _ => None,
    }
}
const PALETTE_SZ: u8 = 12;

fn handler<I, C>(ctx: &'static Context, token: CancellationToken, input: I, cmd: C)
where
    I: Fn(Input) -> Option<bool> + Send + Sync + 'static,
    C: Fn(PaletteColor) -> Command + Send + Sync + 'static
{
    spawn_cancel_from(token, async move {
        let pad = &ctx.pad.as_ref().unwrap();

        let mut held = false;
        let mut rx = pad.subscribe();
        loop {
            use tokio::sync::broadcast::error::RecvError;
            match rx.recv().await {
                Ok(i) => {
                    if let Some(b) = input(i) {
                        held = b;
                        let show = ctx.state.get().await.pause_enable;

                        ctx.state.send(Command::Block(b)).await;
                        if show {
                            ctx.state.send(Command::Pause(b)).await;
                        } else if !b {
                            ctx.state.send(Command::Pause(false)).await;
                        }

                        if show {
                            for i in 0..=12 {
                                if b {
                                    pad.send(Output::Light(Index(i).into(), palette(i as u8).unwrap())).await;
                                } else {
                                    pad.send(Output::Off(Index(i).into())).await;
                                }
                            }
                        }

                    }

                    match i {
                        Input::Press(i, _) => {
                            if held {
                                if let Some(c) = palette(i.0 as u8) {
                                    ctx.state.send(cmd(c)).await;
                                }
                            }
                        },
                        _ => {},
                    }
                },
                Err(RecvError::Lagged(n)) => log::warn!("color select rx lagged by {}", n),
                Err(RecvError::Closed) => break,
            }
        }
    });
}

pub fn launch(ctx: &'static Context) -> CancellationToken {
    let token = CancellationToken::new();

    handler(ctx, token.clone(), |i| match i { Input::Record(b) => Some(b), _ => None }, |c| Command::PrimaryColor(c));
    handler(ctx, token.clone(), |i| match i { Input::Solo(b) => Some(b), _ => None }, |c| Command::SecondaryColor(c));
    handler(ctx, token.clone(), |i| match i { Input::Mute(b) => Some(b), _ => None }, |c| Command::AccentColor(c));

    token
}
