#![feature(async_closure)]

use anyhow::Result;

use tokio::sync::mpsc;
use tokio::task::spawn;

use std::time::Duration;
use tokio::time::sleep;

use rand::Rng;
use std::collections::VecDeque;

use stagebridge::midi::device::launchpad_x::{types::*, *};
use stagebridge::util::future::Broadcast;

mod lights;
mod beatgrid;
mod state;
mod context;
use context::Context;

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();
    let ctx = Context::new().await;
    let pad = ctx.pad.as_ref().unwrap();

    enum Direction {
        Left,
        Right,
        Up,
        Down,
    }

    use std::sync::{Arc, RwLock};
    let mut dir = Arc::new(RwLock::new(Direction::Right));

    let _dir = dir.clone();
    spawn(async move {
        let mut rx = pad.subscribe();
        while let Ok(input) = rx.recv().await {
            match input {
                Input::Up(true) => { *_dir.write().unwrap() = Direction::Up; },
                Input::Down(true) => { *_dir.write().unwrap() = Direction::Down; },
                Input::Left(true) => { *_dir.write().unwrap() = Direction::Left; },
                Input::Right(true) => { *_dir.write().unwrap() = Direction::Right; },
                _ => {}
            }
        }
    });

    let mut snake = VecDeque::with_capacity(64);

    let mut rng = rand::thread_rng();
    let mut rand_pos = move || Coord(rng.gen_range(0..8), rng.gen_range(0..8));

    for i in 0..64 {
        pad.send(Output::Light(Index(i).into(), Color::Index(1))).await;
    }
    pad.send(Output::Light(Coord(0, 8).into(), Color::Index(41))).await;
    pad.send(Output::Light(Coord(1, 8).into(), Color::Index(41))).await;
    pad.send(Output::Light(Coord(2, 8).into(), Color::Index(41))).await;
    pad.send(Output::Light(Coord(3, 8).into(), Color::Index(41))).await;

    let mut fruit = rand_pos();
    pad.send(Output::Light(fruit.into(), Color::Index(5))).await;

    let start = Coord(0, 0);
    snake.push_front(start);
    pad.send(Output::Light(start.into(), Color::Index(21))).await;

    loop {
        let Coord(mut x, mut y) = snake.front().unwrap();
        let (dx, dy) = match *dir.read().unwrap() {
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
            Direction::Up => (0, 1),
            Direction::Down => (0, -1),
        };

        x += dx;
        if x > 7 {
            x = 0;
        }
        if x < 0 {
            x = 7;
        }

        y += dy;
        if y > 7 {
            y = 0;
        }
        if y < 0 {
            y = 7;
        }


        let head = Coord(x, y);
        snake.push_front(head);
        pad.send(Output::Light(head.into(), Color::Index(21))).await;

        if head != fruit {
            let tail = snake.pop_back().unwrap();
            pad.send(Output::Light(tail.into(), Color::Index(1))).await;
        }

        if head == fruit {
            fruit = rand_pos();
            let mut good = true;
            while !good {
                for c in &snake {
                    good = good && fruit != *c;
                }

                if !good {
                    fruit = rand_pos();
                }
            }
            pad.send(Output::Light(fruit.into(), Color::Index(5))).await;
        }

        sleep(Duration::from_millis(250)).await;
    }

    Ok(())
}
