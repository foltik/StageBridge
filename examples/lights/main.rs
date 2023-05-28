#![feature(async_closure)]

#![allow(dead_code)]

use anyhow::Result;

// use std::time::Duration;

use tokio::task::spawn;
// use tokio::time::sleep;
// use tokio::sync::mpsc;

// use std::sync::Arc;
// use parking_lot::Mutex;
use tokio_util::sync::CancellationToken;

use stagebridge::util::future::Broadcast;
// use stagebridge::util::pipeline::Pipeline;

mod beatgrid;
mod lights;

mod context;
pub use context::Context;

mod state;
pub use state::*;

#[allow(dead_code)]
mod pipeline;

mod fx;

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();
    let ctx = Context::new().await;

    if let Some(pad) = ctx.pad.as_ref() {
        use stagebridge::midi::device::launchpad_x::{*, types::*};
        // use pipeline::{logic, light, time, map, filter};
        use pipeline::logic;

        logic::beat_indicator::launch(ctx);

        logic::div_select::launch(ctx);
        logic::active_toggle::launch(ctx);
        logic::palette_switcher::launch(ctx);
        logic::pause_enable::launch(ctx);
        logic::color_select::launch(ctx);
        
        // logic::pad_alpha::launch(ctx);

        spawn(async move {
            let mut token = CancellationToken::new();

            let mut rx = ctx.state.subscribe_mode();
            loop {
                use tokio::sync::broadcast::error::RecvError;
                match rx.recv().await {
                    Ok(state) => {
                        token.cancel();
                        pad.send(Output::Brightness(1.0)).await;

                        use state::Mode;
                        token = match state.mode {
                            Mode::Off => fx::solid(ctx, PaletteColor::Off),
                            Mode::Manual => fx::manual(ctx, state.primary),

                            Mode::Solid => fx::solid(ctx, state.primary),
                            Mode::SolidHue => fx::solid_hue(ctx),

                            Mode::Bounce => fx::bounce(ctx, state.primary),
                            Mode::BounceHue => fx::bounce_hue(ctx),

                            Mode::SawUp => fx::saw(ctx, state.primary, true),
                            Mode::SawDown => fx::saw(ctx, state.primary, false),
                            Mode::SawUpHue => fx::saw_hue(ctx, true),
                            Mode::SawDownHue => fx::saw_hue(ctx, false),

                            Mode::Chase => fx::chase(ctx, state.primary, state.secondary),

                            Mode::Strobe => fx::strobe(ctx, state.primary),

                            _ => fx::solid(ctx, PaletteColor::Off),
                            // Mode::Solid2 => todo!(),
                            // Mode::Solid4 => todo!(),
                            // Mode::Bounce { min } => todo!(),
                            // Mode::BounceFlash => todo!(),
                            // Mode::Triangle => todo!(),
                            // Mode::Saw => todo!(),
                            // Mode::Hue => todo!(),
                            // Mode::HueRotate => todo!(),
                            // Mode::Strobe => todo!(),
                            // Mode::Chase => todo!(),
                            // Mode::ChaseHalf => todo!(),
                            // Mode::Alternate => todo!(),
                            // Mode::AlternateFlash => todo!(),
                        };
                    },
                    Err(RecvError::Lagged(n)) => log::warn!("main state recv lagged by {}", n),
                    Err(RecvError::Closed) => log::error!("main state recv closed"),
                }
            }
        });

        let process = async move |x, y| {
            use state::Mode;
            if let Some((div, mode)) = match (x, y) {
                (0, 2) => Some(((1, 32), Mode::Strobe)),
                (1, 2) => Some(((1, 16), Mode::Strobe)),
                (2, 2) => Some(((1, 8), Mode::Strobe)),
                (3, 2) => Some(((1, 4), Mode::Strobe)),
                (4, 2) => Some(((1, 16), Mode::Chase)),
                (5, 2) => Some(((1, 8), Mode::Chase)),
                (6, 2) => Some(((1, 4), Mode::Chase)),
                (7, 2) => Some(((1, 2), Mode::Chase)),

                (0, 4) => Some(((1, 8), Mode::Bounce)),
                (1, 4) => Some(((1, 4), Mode::Bounce)),
                (2, 4) => Some(((1, 2), Mode::Bounce)),
                (3, 4) => Some(((1, 1), Mode::Bounce)),
                (4, 4) => Some(((1, 8), Mode::BounceHue)),
                (5, 4) => Some(((1, 4), Mode::BounceHue)),
                (6, 4) => Some(((1, 2), Mode::BounceHue)),
                (7, 4) => Some(((1, 1), Mode::BounceHue)),

                (0, 6) => Some(((1, 8), Mode::SawUp)),
                (1, 6) => Some(((1, 4), Mode::SawUp)),
                (2, 6) => Some(((1, 2), Mode::SawUp)),
                (3, 6) => Some(((1, 1), Mode::SawUp)),
                (4, 6) => Some(((1, 8), Mode::SawUpHue)),
                (5, 6) => Some(((1, 4), Mode::SawUpHue)),
                (6, 6) => Some(((1, 2), Mode::SawUpHue)),
                (7, 6) => Some(((1, 1), Mode::SawUpHue)),

                (0, 7) => Some(((1, 8), Mode::SawDown)),
                (1, 7) => Some(((1, 4), Mode::SawDown)),
                (2, 7) => Some(((1, 2), Mode::SawDown)),
                (3, 7) => Some(((1, 1), Mode::SawDown)),
                (4, 7) => Some(((1, 8), Mode::SawDownHue)),
                (5, 7) => Some(((1, 4), Mode::SawDownHue)),
                (6, 7) => Some(((1, 2), Mode::SawDownHue)),
                (7, 7) => Some(((1, 1), Mode::SawDownHue)),

                // (4, 4) => Some(Mode::ChaseHalf),
                // (4, 3) => Some(Mode::ChaseRotate),
                // (5, 5) => Some(Mode::Alternate),
                // (5, 4) => Some(Mode::AlternateFlash),
                // (6, 4) => Some(Mode::StrobeAccent),
                
                _ => None,
            } {
                ctx.state.send(Command::Div(div)).await;
                ctx.state.send(Command::Mode(mode)).await;
            }

            if let Some(mode) = match (x, y) {
                (0, 0) => Some(Mode::Off),
                (1, 0) => Some(Mode::Solid),
                (2, 0) => Some(Mode::SolidHue),
                _ => None
            } {
                ctx.state.send(Command::Mode(mode)).await;
            }

            match (x, y) {
                (6, 0) => ctx.state.send(Command::PrimaryColor(PaletteColor::White)).await,
                (7, 0) => {
                    let old = ctx.state.get().await.primary;

                    let mut primary: PaletteColor = rand::random();
                    while primary == old {
                        primary = rand::random();
                    }

                    let mut secondary: PaletteColor = rand::random();
                    while secondary == primary {
                        secondary = rand::random();
                    }

                    ctx.state.send(Command::PrimaryColor(primary)).await;
                    ctx.state.send(Command::SecondaryColor(secondary)).await;
                }
                _ => {},
            }
        };

        spawn(async move {
            let mut rx = pad.subscribe();
            loop {
                use tokio::sync::broadcast::error::RecvError;
                match rx.recv().await {
                    Ok(input) => { 
                        if ctx.state.get().await.block { continue }

                        // log::debug!("{:?}", input);

                        match input {
                            Input::Press(pos, _) => {
                                let pos = Pos::from(pos);
                                let Coord(x, y) = pos.into();

                                process(x, y).await;
                            },
                            _ => {},
                        }


                    },
                    Err(RecvError::Lagged(n)) => log::warn!("main input rx lagged by {}", n),
                    Err(RecvError::Closed) => return,
                }
            }
        });

        ctx.osc.spawn(async move |osc| {
            let mut rx = osc.subscribe();
            loop {
                use tokio::sync::broadcast::error::RecvError;
                use stagebridge::osc::{Message, Value};
                match rx.recv().await {
                    Ok(Message { addr, args }) => match addr.as_str() {
                        "/input" => {
                            if let Value::Int(y) = args[0] {
                                if let Value::Int(x) = args[1] {

                                    log::debug!("{}, {}", x, y);
                                    process(x as i8, y as i8).await;
                                }
                            }
                        }
                        _ => {}
                    },
                    Err(RecvError::Lagged(n)) => log::warn!("osc input rx lagged by {}", n),
                    Err(RecvError::Closed) => return,
                }
            }
        });

        for i in 0..7 {
            pad.send(Output::Light(Coord(i, 8).into(), PaletteColor::Index(1))).await;
            pad.send(Output::Light(Coord(8, i).into(), PaletteColor::Index(1))).await;
        }
        ctx.state.send(Command::Active(true)).await;
        ctx.state.send(Command::Palette(Palette::Main)).await;
        ctx.state.send(Command::Div((1, 4))).await;
    }

    if let Some(ctrl) = ctx.ctrl.as_ref() {
        ctrl.listen(async move |i| {
            use stagebridge::midi::device::worlde_easycontrol9::*;

            if let Some(cmd) = match i {
                Input::Slider(0, f) => Some(Command::Duty(f)),
                Input::Slider(1, f) => Some(Command::Min(f)),
                Input::Slider(2, f) => Some(Command::Max(f)),

                Input::Knob(8, f) => Some(Command::Alpha(f)),
                Input::Slider(8, f) => Some(Command::OffsetLight(f)),
                Input::Slider(7, f) => Some(Command::OffsetPad(f)),
                _ => None,
            } {
                ctx.state.send(cmd).await;
            }
        });
    }

    std::thread::park();
    Ok(())
}


        // spawn(async move {
        //     let _pipe = Pipeline::<Input>::new()
        //         .chain(filter::press())
        //         .chain(light::shape::rect(4, 3))
        //         // .chain(light::shape::row())
        //         // .chain(light::shape::col())
        //         // .chain(light::shape::full())
        //         .chain(time::stagger(Duration::from_millis(100)))
        //         .chain(filter::clamp8())
        //         .map(|p| Output::Light(p, PaletteColor::White))
        //         .chain(map::cancel_off());

        //     // pad.bind_press_release(pipe);
        // });
