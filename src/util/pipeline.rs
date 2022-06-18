use std::fmt::Debug;
use dyn_clone::DynClone;
use std::any::Any;
use std::marker::PhantomData;

use std::sync::Arc;
use futures::Future;
use std::pin::Pin;
use parking_lot::RwLock;

use tokio::task;
use tokio_util::sync::CancellationToken;
use tokio::sync::mpsc;

type Op = Box<dyn OpFn>;
type OpFuture = Pin<Box<dyn Future<Output = ()> + Send>>;

type ItemSender = mpsc::Sender<Box<dyn Item>>;
type ItemReceiver = mpsc::Receiver<Box<dyn Item>>;

pub trait OpFn: FnOnce(ItemReceiver, ItemSender) -> OpFuture + DynClone + Send + Sync + 'static {}
impl <F: FnOnce(ItemReceiver, ItemSender) -> OpFuture + DynClone + Send + Sync + 'static> OpFn for F {}
dyn_clone::clone_trait_object!(OpFn);

pub trait Item: Any + DynClone + Send + Sync + Debug {
    fn unwrap<T>(boxed: Box<dyn Item>) -> T
    where
        Self: Sized,
        T: Item + Copy 
    {
        *(boxed as Box<dyn Any>).downcast()
            .expect("failed to downcast Item")
    }

    fn wrap(self) -> Box<dyn Item>
    where
        Self: Sized,
    {
        Box::new(self)
    }
}
dyn_clone::clone_trait_object!(Item);

impl<T: Send + Sync + DynClone + Debug + 'static> Item for T {}


impl dyn Item {
}


#[derive(Clone)]
pub struct Pipeline<I, O = I> 
where
    I: Item + Copy,
    O: Item + Copy,
{
    ops: Arc<RwLock<Vec<Op>>>,
    _i: PhantomData<I>,
    _o: PhantomData<O>
}

impl<I, O> Pipeline<I, O> 
where
    I: Item + Copy,
    O: Item + Copy,
{
    pub fn new() -> Self {
        Self {
            ops: Arc::new(RwLock::new(Vec::new())),
            _i: PhantomData,
            _o: PhantomData,
        }
    }

    pub fn with_fn<F, Fr, T>(self, f: F) -> Pipeline<I, T>
    where
        F: FnOnce(Receiver<O>, Sender<T>) -> Fr + Clone + Send + Sync + 'static,
        Fr: Future<Output = ()> + Send + 'static,
        T: Item + Copy,
    {
        let Self { ops, .. } = self;

        ops.write().push(Box::new(move |rx, tx| Box::pin(f(Receiver::new(rx), Sender::new(tx)))));

        Pipeline {
            ops,
            _i: PhantomData,
            _o: PhantomData,
        }
    }

    pub fn with_cancel<F, Fr, C, Cr, T>(self, f: F, cancel: C) -> Pipeline<I, T>
    where
        F: Fn(O, Sender<T>) -> Fr + Copy + Send + Sync + 'static,
        Fr: Future<Output = ()> + Send + 'static,
        C: Fn(O, Sender<T>) -> Cr + Copy + Send + Sync + 'static,
        Cr: Future<Output = ()> + Send + 'static,
        T: Item + Copy,
    {
        self.with_fn(async move |mut rx, tx| {
            let token = CancellationToken::new();

            while let Some(o) = rx.recv().await {
                let token = token.clone();
                let tx = tx.clone();

                task::spawn(async move {
                    tokio::select! {
                        _ = f(o, tx.clone()) => {},
                        _ = token.cancelled() => { cancel(o, tx).await; },
                    }
                });
            }

            token.cancel();
        })
    }

    pub fn with<F, Fr, T>(self, f: F) -> Pipeline<I, T>
    where
        F: Fn(O, Sender<T>) -> Fr + Clone + Send + Sync + 'static,
        Fr: Future<Output = ()> + Send + 'static,
        T: Item + Copy,
    {
        self.with_fn(async move |mut rx, tx| {
            while let Some(o) = rx.recv().await {
                f(o, tx.clone()).await;
            }
        })
    }

    pub fn chain<T>(self, pipeline: Pipeline<O, T>) -> Pipeline<I, T>
    where
        T: Item + Copy,
    {
        self.with_fn(async move |mut rx, tx| {
            let (chain_tx, mut chain_rx) = pipeline.spawn();

            task::spawn(async move {
                while let Some(v) = chain_rx.recv().await {
                    tx.send(v).await;
                }
                log::trace!("receiver exited")
            });

            while let Some(o) = rx.recv().await {
                chain_tx.send(o).await;
            }
        })
    }


    pub fn filter<P>(self, pred: P) -> Self
    where
        P: Fn(O) -> bool + Copy + Send + Sync + 'static,
    {
        self.with(async move |o, tx| {
            if pred(o) {
                tx.send(o).await;
            }
        })
    }

    pub fn filter_map<M, V>(self, mapper: M) -> Pipeline<I, V>
    where
        M: Fn(O) -> Option<V> + Copy + Send + Sync + 'static,
        V: Item + Copy,
    {
        self.with(async move |o, tx| {
            if let Some(v) = mapper(o) {
                tx.send(v).await;
            }
        })
    }

    pub fn map<M, T>(self, mapper: M) -> Pipeline<I, T>
    where
        M: Fn(O) -> T + Copy + Send + Sync + 'static,
        T: Item + Copy,
    {
        self.with(async move |o, tx| {
            tx.send(mapper(o)).await;
        })
    }

    pub fn flat_map<M, Ti, T>(self, mapper: M) -> Pipeline<I, T>
    where
        M: Fn(O) -> Ti + Copy + Send + Sync + 'static,
        Ti: IntoIterator<Item = T>,
        <Ti as IntoIterator>::IntoIter: Send,
        T: Item + Copy,
    {
        self.with(async move |o, tx| {
            let ts = mapper(o).into_iter();
            for t in ts {
                tx.send(t).await;
            }
        })
    }

    pub fn spawn(&self) -> (Sender<I>, Receiver<O>) {
        let (main_tx, mut main_rx) = mpsc::channel(16);

        for op in self.ops.read().iter().cloned() {
            let (tx, mut rx) = mpsc::channel(16);
            std::mem::swap(&mut rx, &mut main_rx);

            task::spawn(op(rx, tx.clone()));
        }

        (Sender::new(main_tx), Receiver::new(main_rx))
    }
}

#[derive(Clone)]
pub struct Sender<T> 
where
    T: Item + Copy
{
    tx: mpsc::Sender<Box<dyn Item>>,
    _t: PhantomData<T>,
}
impl <T> Sender<T>
where
    T: Item + Copy
{
    fn new(tx: mpsc::Sender<Box<dyn Item>>) -> Self {
        Self {
            tx,
            _t: PhantomData,
        }
    }

    pub async fn send(&self, t: T) {
        match self.tx.send(t.wrap()).await {
            Err(e) => log::warn!("Pipeline context sender dropped value {:?}", e.0),
            _ => {}
        }
    }
}

pub struct Receiver<T>
where
    T: Item + Copy
{
    rx: mpsc::Receiver<Box<dyn Item>>,
    _t: PhantomData<T>,
}
impl <T> Receiver<T>
where
    T: Item + Copy
{
    fn new(rx: mpsc::Receiver<Box<dyn Item>>) -> Self {
        Self {
            rx,
            _t: PhantomData
        }
    }

    pub async fn recv(&mut self) -> Option<T> {
        self.rx.recv().await
            .map(|i| T::unwrap(i))
    }
}