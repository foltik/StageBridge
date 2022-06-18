use std::time::Duration;
use tokio::time::sleep;

use tokio::task::spawn;
use tokio::sync::mpsc;

// use stagebridge::midi::device::launchpad_x::{types::*, *};
use stagebridge::util::pipeline::{Pipeline, Item};

pub fn delay<T: Item + Copy>(delay: Duration) -> Pipeline<T, T> {
    Pipeline::<T>::new()
        .with(async move |t, tx| {
            sleep(delay).await;
            tx.send(t).await;
        })
}

pub fn stagger<T: Item + Copy>(delay: Duration) -> Pipeline<T, T> {
    Pipeline::<T>::new()
        .with_fn(async move |mut rx, tx| {
            let (buffer_tx, mut buffer_rx) = mpsc::unbounded_channel();

            spawn(async move {
                while let Some(t) = buffer_rx.recv().await {
                    // log::debug!("{:?}", t);
                    tx.send(t).await;
                    sleep(delay).await;
                }
            });

            while let Some(t) = rx.recv().await {
                buffer_tx.send(t).unwrap();
            }
        })
}