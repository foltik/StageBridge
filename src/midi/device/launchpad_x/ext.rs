use tokio::task::spawn;
use tokio_util::sync::CancellationToken;
use crate::util::future::spawn_cancel;

use super::LaunchpadX;

use crate::util::pipeline::Pipeline;
use super::{Input, Output};

use crate::midi::Midi;
use crate::util::future::Broadcast;

pub trait LaunchpadMidiExt {
    fn bind(&self, pipeline: Pipeline<Input, Output>) -> CancellationToken;
    fn bind_press_release(&self, pipeline: Pipeline<Input, Output>) -> CancellationToken;
    fn bind_oneshot(&self, pipeline: Pipeline<Input, Output>) -> CancellationToken;
}

impl LaunchpadMidiExt for Midi<LaunchpadX> {
    // TODO: Use spawn_cancel
    fn bind(&self, pipeline: Pipeline<Input, Output>) -> CancellationToken {
        let token = CancellationToken::new();
        let (tx, mut rx) = pipeline.spawn();

        let mut input = self.subscribe();
        let _token = token.clone();
        spawn(async move {
            tokio::select! {
                _ = async move {
                    loop {
                        use tokio::sync::broadcast::error::RecvError;
                        match input.recv().await {
                            Ok(i) => tx.send(i).await,
                            Err(RecvError::Closed) => break,
                            _ => {}
                        }
                    }
                } => {},
                _ = _token.cancelled() => {}
            }
        });

        let output = self.out_tx.clone();
        spawn(async move {
            while let Some(o) = rx.recv().await {
                output.send(o).await.unwrap();
            }
        });

        token
    }

    fn bind_press_release(&self, pipeline: Pipeline<Input, Output>) -> CancellationToken {
        let _self = self.clone();

        let mut input = self.subscribe();
        let output = self.out_tx.clone();

        spawn_cancel(async move {
            loop {
                let _self = _self.clone();

                use tokio::sync::broadcast::error::RecvError;
                match input.recv().await {
                    Ok(Input::Press(i, f)) => {
                        let (tx, mut rx) = pipeline.spawn();

                        let output = output.clone();
                        spawn(async move {
                            while let Some(o) = rx.recv().await {
                                output.send(o).await.unwrap();
                            }
                        });

                        spawn(async move {
                            tx.send(Input::Press(i, f)).await;

                            let mut sub_input = _self.subscribe();
                            loop {
                                match sub_input.recv().await {
                                    Ok(Input::Release(ir)) => {
                                        if ir == i {
                                            break;
                                        }
                                    },
                                    Err(RecvError::Closed) => break,
                                    _ => {}
                                }
                            }
                        });
                    },
                    Err(RecvError::Closed) => break,
                    _ => {}
                }
            }
        })
    }

    // TODO: Use spawn_cancel
    fn bind_oneshot(&self, pipeline: Pipeline<Input, Output>) -> CancellationToken {
        let token = CancellationToken::new();

        let mut input = self.subscribe();
        let output = self.out_tx.clone();

        let pipeline = pipeline.clone();
        let _token = token.clone();
        spawn(async move {
            tokio::select! {
                _ = async move {
                    loop {
                        use tokio::sync::broadcast::error::RecvError;
                        match input.recv().await {
                            Ok(i) => {
                                let (tx, mut rx) = pipeline.spawn();

                                tx.send(i).await;

                                let output = output.clone();
                                spawn(async move {
                                    while let Some(o) = rx.recv().await {
                                        output.send(o).await.unwrap();
                                    }
                                });
                            },
                            Err(RecvError::Closed) => break,
                            _ => {}
                        }
                    }
                } => {},
                _ = _token.cancelled() => {}
            }
        });

        token
    }
}
