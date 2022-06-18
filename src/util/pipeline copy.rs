use std::fmt::Debug;
use std::marker::PhantomData;

use futures::Future;
use std::pin::Pin;
use std::sync::Arc;

use tokio::task;
use tokio_util::sync::CancellationToken;
use tokio::sync::broadcast;


type Op<T> = Box<Arc<OpFn<T>>>;
type OpFn<T> = dyn Fn(Context<T>) -> OpFuture + Send + Sync + 'static;
type OpFuture = Pin<Box<dyn Future<Output = ()> + Send>>;

pub struct Pipeline<T: Copy + Send + Debug + 'static> {
    ops: Vec<Op<T>>,
    _t: PhantomData<T>,
}

impl<T: Copy + Send + Debug> Pipeline<T> {
    pub fn new() -> Self {
        Self {
            ops: vec![],
            _t: t
        }
    }

    // Add an operation to the pipeline
    pub fn with<F, Fut>(mut self, f: F) -> Self
    where
        F: Fn(Context<T>) -> Fut + Copy + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static
    {
        self.ops.push(Box::new(Arc::new(move |ctx| Box::pin(f(ctx)))));
        self
    }

    pub fn filter<P>(self, pred: P) -> Self
    where
        P: Fn(T) -> bool + Copy + Send + Sync + 'static,
    {
        self.with(async move |mut ctx| {
            while let Some(v) = ctx.recv().await {
                if pred(v) {
                    ctx.send(v);
                }
            }
        })
    }

    pub fn map<M>(self, mapper: M) -> Self
    where
        M: Fn(T) -> T + Copy + Send + Sync + 'static,
    {
        self.with(async move |mut ctx| {
            while let Some(v) = ctx.recv().await {
                ctx.send(mapper(v));
            }
        })
    }

    pub fn flat_map<M, I>(self, mapper: M) -> Self
    where
        M: Fn(T) -> I + Copy + Send + Sync + 'static,
        I: IntoIterator<Item = T>
    {
        self.with(async move |mut ctx| {
            while let Some(v) = ctx.recv().await {
                for v in mapper(v) {
                    ctx.send(v);
                }
            }
        })
    }

    // Spawn a task for each operation, linking them up with
    // broadcast channels for input, between each operation,
    // and for output.
    pub fn run(&self) -> Instance<T> {
        let (main_tx, mut rx) = broadcast::channel(16);

        let token = CancellationToken::new();
        let mut parent_tx = main_tx.clone();

        for op in self.ops.iter().cloned() {
            let (tx, mut parent_rx) = broadcast::channel(16);
            std::mem::swap(&mut rx, &mut parent_rx);

            let _parent_tx = parent_tx.clone();
            parent_tx = tx.clone();

            let ctx = Context {
                parent_tx: _parent_tx,
                parent_rx,
                tx,
                token: token.clone(),
            };

            ctx.spawn(async move |ctx| op(ctx).await);
        }

        Instance {
            tx: main_tx,
            rx,
            token,
        }
    }
}

pub struct Instance<T: Copy + Send + Debug + 'static> {
    tx: broadcast::Sender<T>,
    rx: broadcast::Receiver<T>,
    token: CancellationToken,
}

impl<T: Copy + Send + Debug> Instance<T> {
    pub fn send(&self, value: T) {
        match self.tx.send(value) {
            Err(e) => log::warn!("Pipeline sender dropped value {:?}", e.0),
            _ => {},
        }
    }

    pub async fn recv(&mut self) -> Option<T> {
        use tokio::sync::broadcast::error::RecvError;
        loop {
            match self.rx.recv().await {
                Ok(v) => return Some(v),
                Err(RecvError::Lagged(n)) => {
                    log::warn!("Pipeline receiver lagged by {}", n);
                    continue;
                }
                Err(RecvError::Closed) => return None
            }
        }
    }

    pub fn stop(self) {
        self.token.cancel();
    }
}

pub struct Context<T: Copy + Send + 'static> {
    parent_tx: broadcast::Sender<T>,
    parent_rx: broadcast::Receiver<T>,
    tx: broadcast::Sender<T>,
    token: CancellationToken,
}

impl <T: Copy + Send + Debug> Context<T> {
    pub fn spawn<F, Fut>(&self, f: F)
    where 
        F: FnOnce(Context<T>) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static
    {
        let _self = self.clone();
        let _token = self.token.clone();
        task::spawn(async move {
            tokio::select! {
                _ = _token.cancelled() => {}
                _ = f(_self) => {}
            };
        });
    }

    fn clone(&self) -> Self {
        Self {
            parent_tx: self.parent_tx.clone(),
            parent_rx: self.parent_tx.subscribe(),
            tx: self.tx.clone(),
            token: self.token.clone()
        }
    }

    pub fn send(&self, value: T) {
        match self.tx.send(value) {
            Err(e) => log::warn!("Pipeline context sender dropped value {:?}", e.0),
            _ => {},
        }
    }

    pub async fn recv(&mut self) -> Option<T> {
        use tokio::sync::broadcast::error::RecvError;
        loop {
            match self.parent_rx.recv().await {
                Ok(v) => return Some(v),
                Err(RecvError::Lagged(n)) => {
                    log::warn!("Pipeline context receiver lagged by {}", n);
                    continue;
                }
                Err(RecvError::Closed) => return None
            }
        }
    }
}