use anyhow::Result;

use std::time::Duration;

use tokio::task;
use tokio::sync::mpsc;

use stagebridge::midi::{Midi};
use stagebridge::midi::device::launchpad_x::{*, gen::*, peepline::*, types::*};

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let pad = Midi::<LaunchpadX>::open("Launchpad X:Launchpad X MIDI 2").unwrap();

    pad.send(Output::Mode(Mode::Programmer)).await;
    pad.send(Output::Clear).await;
 
    let _pad = pad.clone();
    task::spawn(async move {
        let mut pipeline = Pipeline::new()
            .dupe(|p| [p.shift_y(2), p.shift_y(4)])
            .filter_8()
            .flat_map_pos(|p| Diagonal::new(p))
            .flat_map(|Event { pos, color, delay }| [
                Event { pos, color, delay },
                Event { pos, color: PaletteColor::Off, delay: Duration::from_millis(500).into() }
            ]);
            // .with(|prev, next| {

            //     prev.iter()
            //         .for_each(|Event { pos, color, delay: _ }| {
            //             let coord = Coord::from(pos);
            //             for i in 0..5 {
            //                 let c = Coord(coord.0 + i, coord.1 + i);

            //                 if c.0 < 8 && c.1 < 8 {
            //                     next.push(Event { 
            //                         pos: c.into(),
            //                         color,
            //                         delay: Duration::from_millis(50 * i as u64).into() 
            //                     });

            //                     next.push(Event { 
            //                         pos: c.into(),
            //                         color: Color::Off,
            //                         delay: Duration::from_millis(250 + 50 * i as u64).into() 
            //                     });
            //                 }
            //             }
            //         });
            // });

        let mut rx = _pad.subscribe();
        while let Ok(input) = rx.recv().await {
            log::debug!("{:?}", input);

            if let Input::Release(i) = input {
            }

            if let Input::Press(i, _v) = input {
                let pos = Pos::from(i);
                
                for e in pipeline.run(pos, PaletteColor::White) {
                    if e.delay.is_zero() {
                        _pad.send(Output::Light(e.pos, e.color)).await;
                    } else {
                        let _pad = _pad.clone();
                        task::spawn(async move {
                            tokio::time::sleep(e.delay.duration(120.0)).await;
                            _pad.send(Output::Light(e.pos, e.color)).await;
                        });
                    }
                }

                if pos == Coord(0, 0).into() {
                    // for e in pipeline.run
                }

                // for p in gen::Horizontal::new(pos) {
                //     _pad.send(Output::Light(p, Color::White)).await;
                // }

                // for p in gen::Vertical::new(pos) {
                //     _pad.send(Output::Light(p, Color::White)).await;
                // }

                // if pos == Coord(0, 0).into() {
                //     let _pad = _pad.clone();
                //     task::spawn(async move {
                //         for x in 0..8 {
                //             for i in 0..(x + 1) {
                //                 _pad.send(Output::Light(Coord(x - i, i).into(), Color::White)).await;
                //             }
                //             tokio::time::sleep(Duration::from_millis(100)).await;
                //         }

                //         for y in 1..8 {
                //             for i in 0..(8 - y) {
                //                 _pad.send(Output::Light(Coord(7 - i, y + i).into(), Color::White)).await;
                //                 log::debug!("{}, {}", 7 - i, i + 1);
                //             }
                //             tokio::time::sleep(Duration::from_millis(100)).await;
                //         }
                //     });
                // }
            }
        }
    });

    enum Message {
        Bpm(u32),
        Stop,
        Start,
    }

    fn clock(bpm: u32) -> Duration {
        let beat = 1.0 / ((bpm as f32) / 60.0);
        Duration::from_secs_f32(beat / 24.0)
    }

    let (msg_tx, mut msg_rx) = mpsc::channel::<Message>(8);

    let _pad = pad.clone();
    task::spawn(async move {
        let mut interval = tokio::time::interval(clock(120));
        let mut run = false;

        let mut n = 0;

        loop {
            interval.tick().await;

            match msg_rx.try_recv() {
                Ok(msg) => match msg {
                    Message::Bpm(n) => interval = tokio::time::interval(clock(n)),
                    Message::Stop => {
                        run = false;
                    },
                    Message::Start => {
                        run = true;
                    },
                },
                _ => {}
            }

            if run {
                _pad.send(Output::Clock).await;

                if n % 2 == 0 {
                    for i in 0..64 {
                        _pad.send(Output::Light(Index(i).into(), PaletteColor::White)).await;
                    }
                } else {
                    for i in 0..64 {
                        _pad.send(Output::Off(Index(i).into())).await;
                    }
                }
            }

            n += 1;
        }
    });

    loop {
        let mut cmd = String::new();
        std::io::stdin().read_line(&mut cmd).unwrap();

        let ch = cmd.chars().collect::<Vec<_>>();
        let end = cmd.len() - 2;

        if cmd.len() > 0 {
            match ch[0] {
                'm' => match ch[1] {
                    'l' => pad.send(Output::Mode(Mode::Live)).await,
                    'p' => pad.send(Output::Mode(Mode::Programmer)).await,
                    _ => {}
                },
                'b' => {
                    let fr = cmd[1..=end].parse::<f32>().unwrap();
                    pad.send(Output::Brightness(fr)).await;
                },
                'a' => {
                    let mode = match ch[1] {
                        'm' => Pressure::Channel,
                        'p' => Pressure::Polyphonic,
                        'o' => Pressure::Off,
                        _ => Pressure::Channel,
                    };
                    let thres = match ch[2] {
                        'l' => PressureCurve::Low,
                        'm' => PressureCurve::Medium,
                        'h' => PressureCurve::High,
                        _ => PressureCurve::Medium
                    };
                    pad.send(Output::Pressure(mode, thres)).await;
                },
                'v' => {
                    let v = match cmd.len() {
                        3 => 0,
                        _ => {
                            log::debug!("'{}'", &cmd[2..=end]);
                            cmd[2..=end].parse::<u8>().unwrap()
                        },
                    };
                    let velocity = match ch[1] {
                        'l' => Velocity::Low,
                        'm' => Velocity::Medium,
                        'h' => Velocity::High,
                        'f' => Velocity::Fixed(v),
                        _ => Velocity::Medium
                    };
                    pad.send(Output::Velocity(velocity)).await;
                },
                'i' => pad.send(Output::Light(Index(cmd[1..=end].parse::<i8>().unwrap()).into(), PaletteColor::White)).await,
                'o' => pad.send(Output::Off(Index(cmd[1..=end].parse::<i8>().unwrap()).into())).await,
                'f' => pad.send(Output::Flash(Index(cmd[1..=end].parse::<i8>().unwrap()).into(), PaletteColor::White)).await,
                'p' => pad.send(Output::Pulse(Index(cmd[1..=end].parse::<i8>().unwrap()).into(), PaletteColor::White)).await,
                'I' => {
                    let _pad = pad.clone();
                    task::spawn(async move {
                        for i in 0..64 {
                            _pad.send(Output::Light(Index(i).into(), PaletteColor::Index((63 - i as u8) + 64))).await;
                            tokio::time::sleep(Duration::from_millis(10)).await;
                        }
                    });
                },
                'J' => {
                    let _pad = pad.clone();
                    task::spawn(async move {
                        for i in 0..64 {
                            _pad.send(Output::Light(Index(i).into(), PaletteColor::Index(i as u8))).await;
                            tokio::time::sleep(Duration::from_millis(10)).await;
                        }
                    });
                }
                'O' => {
                    let _pad = pad.clone();
                    task::spawn(async move {
                        for i in 0..64 {
                            _pad.send(Output::Off(Index(i).into())).await;
                            tokio::time::sleep(Duration::from_millis(5)).await;
                        }
                    });
                },
                'Z' => {
                    let _pad = pad.clone();
                    task::spawn(async move {
                        let mut c = 0u8;
                        loop {
                            c = (c + 1) % 127;
                            tokio::time::sleep(Duration::from_millis(10)).await;

                            for i in 0..64 {
                                _pad.send(Output::Rgb(Index(i).into(), (c, c, c))).await;
                            }
                        }
                    });
                },
                'B' => msg_tx.send(Message::Bpm(cmd[1..=end].parse::<u32>().unwrap())).await.map_err(|_| ()).unwrap(),
                'S' => msg_tx.send(Message::Start).await.map_err(|_| ()).unwrap(),
                's' => msg_tx.send(Message::Stop).await.map_err(|_| ()).unwrap(),
                _ => {}
            }
        }
    }
}
