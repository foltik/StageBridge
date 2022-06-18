use std::time::Duration;
use tokio::time::sleep;

use futures::Future;
use tokio::task::spawn;
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;

/// Spawns future that runs another future if the operation
/// is cancelled by the provided token before it completes.
pub fn spawn_fn_cancel_from<Fr, Cr>(token: CancellationToken, future: Fr, on_cancel: Cr)
where
    Fr: Future<Output = ()> + Send + 'static,
    Cr: Future<Output = ()> + Send + 'static
{
    spawn(async move {
        tokio::select! {
            _ = future => {},
            _ = token.cancelled() => {
                on_cancel.await;
            },
        }
    });
}

pub fn spawn_cancel_from<Fr>(token: CancellationToken, future: Fr)
where
    Fr: Future<Output = ()> + Send + 'static,
{
    spawn_fn_cancel_from(token, future, async {})
}

pub fn spawn_fn_cancel<Fr, Cr>(future: Fr, on_cancel: Cr) -> CancellationToken 
where
    Fr: Future<Output = ()> + Send + 'static,
    Cr: Future<Output = ()> + Send + 'static,
{
    let token = CancellationToken::new();
    spawn_fn_cancel_from(token.clone(), future, on_cancel);
    token
}

pub fn spawn_cancel<Fr>(future: Fr) -> CancellationToken 
where
    Fr: Future<Output = ()> + Send + 'static,
{
    let token = CancellationToken::new();
    spawn_cancel_from(token.clone(), future);
    token
}

pub fn spawn_interval<F, Fr>(f: F, pd: Duration) -> CancellationToken
where
    F: Fn() -> Fr + Send + Sync + 'static,
    Fr: Future<Output = ()> + Send + 'static
{
    spawn_cancel(async move {
        loop {
            f().await;
            sleep(pd).await;
        }
    })
}

pub trait Broadcast<T: Clone + Send + 'static> {
    fn subscribe(&self) -> broadcast::Receiver<T>;

    fn listen<F, Fr>(&self, f: F) -> CancellationToken
    where
        F: FnOnce(T) -> Fr + Clone + Send + Sync + 'static,
        Fr: Future<Output = ()> + Send + 'static,
    {
        let token = CancellationToken::new();

        let _token = token.clone();
        let mut rx = self.subscribe();
        // TODO: spawn_cancel
        spawn(async move {
            loop {
                // log::debug!("rx looping");
                tokio::select! {
                    res = rx.recv() => {
                        use broadcast::error::RecvError;
                        match res {
                            Ok(t) => { spawn(f.clone()(t)); },
                            Err(RecvError::Lagged(n)) => log::warn!("broadcast rx lagged by {}", n),
                            Err(RecvError::Closed) => return,
                        }
                    }
                    _ = _token.cancelled() => return
                };
            }
        });

        token
    }

    // pub fn listen_where<P, F, R>(p: P, f: F)
    // where
    //     F: Fn(T) -> R + Send + Sync + 'static,
    //     R: Future<Output = ()> + Send + 'static,

    fn listen_oneshot<P, F, R>(&self, p: P, f: F) -> CancellationToken
    where
        P: Fn(&T) -> bool + Send + Sync + 'static,
        F: Fn(T) -> R + Send + Sync + 'static,
        R: Future<Output = ()> + Send + 'static,
    {
        let token = CancellationToken::new();

        let _token = token.clone();
        let mut rx = self.subscribe();
        spawn(async move {
            loop {
                tokio::select! {
                    res = rx.recv() => {
                        use broadcast::error::RecvError;
                        match res {
                            Ok(t) => { 
                                if p(&t) {
                                    spawn(f(t)); 
                                    return;
                                }
                            },
                            Err(RecvError::Lagged(n)) => log::warn!("broadcast rx lagged by {}", n),
                            Err(RecvError::Closed) => return,
                        }
                    }
                    _ = _token.cancelled() => return
                };
            }
        });

        token
    }
}