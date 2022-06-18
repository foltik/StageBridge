use std::fmt::Debug;
use dyn_clone::DynClone;
use std::any::Any;
use std::marker::PhantomData;

use futures::Future;
use std::pin::Pin;
use std::sync::Arc;

use tokio::task;
use tokio_util::sync::{CancellationToken, WaitForCancellationFuture};
use tokio::sync::mpsc;

type Sender = mpsc::Sender<Box<dyn Item>>;
type Receiver = mpsc::Receiver<Box<dyn Item>>;

type Op = Box<dyn OpFn>;
// type OpFn = dyn FnOnce(AnyContext) -> OpFuture + Clone + Send + Sync + 'static;
type OpFuture = Pin<Box<dyn Future<Output = ()> + Send>>;

pub trait OpFn: FnOnce(BaseContext) -> OpFuture + DynClone + Send + Sync + 'static {}
impl <F: FnOnce(BaseContext) -> OpFuture + Send + Sync + 'static> OpFn for F {}
dyn_clone::clone_trait_object!(OpFn);

pub trait Item: Any + DynClone + Send + Sync + Debug {}
impl<T: Send + Sync + DynClone + Debug + 'static> Item for T {}
dyn_clone::clone_trait_object!(Item);

#[derive(Clone)]
pub struct Pipeline<I, O = I> 
where
    I: Item + Copy + 'static,
    O: Item + Copy + 'static,
{
    ops: Vec<Op>,
    _i: PhantomData<I>,
    _o: PhantomData<O>
}

impl<I, O> Pipeline<I, O> 
where
    I: Item + Copy + 'static,
    O: Item + Copy + 'static,
{
    pub fn new() -> Self {
        Self {
            ops: vec![],
            _i: PhantomData,
            _o: PhantomData,
        }
    }

    // Add an operation to the pipeline
    pub fn with<F, Fut, V>(self, f: F) -> Pipeline<I, V>
    where
        F: FnOnce(Context<O, V>) -> Fut + Clone + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
        V: Item + Copy + 'static,
    {
        let Self { mut ops, .. } = self;
        ops.push(Box::new(move |ctx| Box::pin(f.clone()(Context::new(ctx)))));

        Pipeline {
            ops,
            _i: PhantomData,
            _o: PhantomData,
        }
    }

    pub fn chain<V>(self, pipeline: Pipeline<O, V>) -> Pipeline<I, V>
    where
        V: Item + Copy + 'static,
    {
        let pipeline = Arc::new(pipeline);
        self.with(async move |ctx| {
            log::debug!("chain start");
            let chain = pipeline.run();

            let token = ctx.token();
            let _chain = chain.clone();
            task::spawn(async move {
                token.cancelled().await;
                log::debug!("chain cancel");
                _chain.cancel();
            });

            let mut _ctx = ctx.clone();
            chain.spawn(async move |mut chain| {
                while let Some(v) = chain.recv().await {
                    _ctx.send(v);
                }
            });

            let mut _ctx = ctx.clone();
            while let Some(o) = _ctx.recv().await {
                chain.send(o);
            }
        })
    }


    pub fn filter<P>(self, pred: P) -> Self
    where
        P: Fn(O) -> bool + Copy + Send + Sync + 'static,
    {
        self.with(async move |mut ctx| {
            while let Some(v) = ctx.recv().await {
                if pred(v) {
                    ctx.send(v);
                }
            }
        })
    }

    pub fn filter_map<M, V>(self, mapper: M) -> Pipeline<I, V>
    where
        M: Fn(O) -> Option<V> + Copy + Send + Sync + 'static,
        V: Item + Copy + 'static,
    {
        self.with(async move |mut ctx| {
            while let Some(o) = ctx.recv().await {
                if let Some(v) = mapper(o) {
                    ctx.send(v);
                }
            }
        })
    }

    pub fn map<M, V>(self, mapper: M) -> Pipeline<I, V>
    where
        M: Fn(O) -> V + Copy + Send + Sync + 'static,
        V: Item + Copy + 'static,
    {
        self.with(async move |mut ctx| {
            while let Some(v) = ctx.recv().await {
                ctx.send(mapper(v));
            }
        })
    }

    pub fn flat_map<M, Vi, V>(self, mapper: M) -> Pipeline<I, V>
    where
        M: Fn(O) -> Vi + Copy + Send + Sync + 'static,
        Vi: IntoIterator<Item = V>,
        V: Item + Copy + 'static,
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
    pub fn run(&self) -> Context<O, I> {
        let (main_tx, mut main_rx) = broadcast::channel(16);

        let (proxy_tx, proxy_rx) = broadcast::channel(16);
        let _proxy_tx = proxy_tx.clone();
        task::spawn(async move {
            log::debug!("proxy rxing");
            while let Ok(v) = main_rx.recv().await {
                _proxy_tx.send(v);
            }
            log::debug!("proxy dropped");
        });

        let token = CancellationToken::new();
        let mut parent_tx = proxy_tx.clone();
        let mut rx = proxy_rx;

        for op in self.ops.iter().cloned() {
            let (tx, mut parent_rx) = broadcast::channel(16);
            std::mem::swap(&mut rx, &mut parent_rx);

            let _parent_tx = parent_tx.clone();
            parent_tx = tx.clone();

            let ctx = BaseContext {
                parent_tx: _parent_tx,
                rx: parent_rx,
                tx,
                token: token.clone(),
            };

            ctx.spawn(async move |ctx| op(ctx).await);
        }

        Context::new(BaseContext {
            tx: main_tx,
            parent_tx,
            rx,
            token,
        })
    }
}

pub struct BaseContext {
    rx: Receiver,
    tx: Sender,
}

impl BaseContext {
    pub fn spawn<F, Fut>(&self, f: F)
    where 
        F: FnOnce(BaseContext) -> Fut + Send + 'static,
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
}

pub struct Context<I, O> 
where
    I: Item + Copy + 'static,
    O: Item + Copy + 'static,
{
    base: BaseContext,
    _i: PhantomData<I>,
    _o: PhantomData<O>,
}

impl <I, O> Context<I, O> 
where
    I: Item + Copy + 'static,
    O: Item + Copy + 'static,
{
    fn new(base: BaseContext) -> Self {
        Context {
            base,
            _i: PhantomData,
            _o: PhantomData,
        }
    }

    fn token(&self) -> CancellationToken {
        self.base.token.clone()
    }

    pub fn send(&self, value: O) {
        match self.base.tx.send(Box::new(value)) {
            Err(e) => log::warn!("Pipeline context sender dropped value {:?}", e.0),
            _ => {},
        }
    }

    pub async fn recv(&mut self) -> Option<I> {
        use tokio::sync::broadcast::error::RecvError;
        loop {
            match self.base.rx.recv().await {
                Ok(v) => return Some(*(v as Box<dyn Any>).downcast().expect("failed to downcast")),
                Err(RecvError::Lagged(n)) => {
                    log::warn!("Pipeline context receiver lagged by {}", n);
                    continue;
                }
                Err(RecvError::Closed) => return None
            }
        }
    }

    pub fn spawn<F, Fut>(&self, f: F)
    where 
        F: FnOnce(Self) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static
    {
        let _self = self.clone();
        let _token = self.base.token.clone();
        task::spawn(async move {
            tokio::select! {
                _ = _token.cancelled() => {}
                _ = f(_self) => {}
            };
        });
    }

    pub fn cancel(self) {
        self.base.token.cancel();
    }

    pub fn cancelled(&self) -> WaitForCancellationFuture {
        self.base.token.cancelled()
    }

    pub fn on_cancel<F, Fut>(&self, f: F)
    where
        F: FnOnce(Self) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static
    {
        let _self = self.clone();
        let token = self.token();
        task::spawn(async move {
            token.cancelled().await;
            f(_self).await;
        });
    }

    pub fn clone(&self) -> Self {
        Self {
            base: self.base.clone(),
            _i: PhantomData,
            _o: PhantomData,
        }
    }
}

pub struct SubContext<I, O> 
where
    I: Item + Copy + 'static,
    O: Item + Copy + 'static,
{
    tx: Sender,
    cancel: CancellationToken,
    _i: PhantomData<I>,
    _o: PhantomData<O>,
}

impl <I, O> Context<I, O> 
where
    I: Item + Copy + 'static,
    O: Item + Copy + 'static,
{

}