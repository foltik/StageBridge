use std::fmt::Debug;
use std::result::Result;
use thiserror::Error;

use parking_lot::Mutex;
use std::sync::Arc;
use futures::Future;
use tokio::sync::{broadcast, mpsc};
use tokio::task;

use midir::{MidiInput, MidiInputConnection, MidiOutput};

pub mod device;

#[derive(Error, Debug)]
pub enum MidiError {
    #[error("failed to initialize MIDI context")]
    InitializationFailed(#[from] midir::InitError),
    #[error("failed to connect to device '{0}'")]
    ConnectionFailed(String),
    #[error("device '{0}' not found")]
    DeviceNotFound(String),
}

const CHAN_SIZE: usize = 128;

#[derive(Clone)]
pub struct Midi<D: Device> {
    state: Arc<Mutex<D>>,

    in_tx: broadcast::Sender<D::Input>,
    out_tx: mpsc::Sender<D::Output>,

    _input: Arc<Mutex<MidiInputConnection<()>>>,
}

impl<D: Device + Clone + Send> Midi<D> {
    pub fn list() -> Result<(), MidiError> {
        let midi_in = MidiInput::new("StageBridge").map_err(|e| MidiError::from(e))?;
        for port in midi_in.ports() {
            log::info!("IN: '{:?}'", midi_in.port_name(&port).unwrap());
        }

        let midi_out = MidiOutput::new("StageBridge").map_err(|e| MidiError::from(e))?;
        for port in midi_out.ports() {
            log::info!("OUT: '{:?}'", midi_out.port_name(&port).unwrap());
        }

        Ok(())
    }

    pub fn open(name: &str) -> Result<Self, MidiError> {
        let state = Arc::new(Mutex::new(D::new()));

        let (in_tx, _in_rx) = broadcast::channel(CHAN_SIZE);
        let (out_tx, mut out_rx) = mpsc::channel(CHAN_SIZE);

        let midi_in =
            MidiInput::new(&format!("StageBridge_in_{}", name)).map_err(|e| MidiError::from(e))?;
        let in_port = midi_in
            .ports()
            .into_iter()
            .find(|p| midi_in.port_name(p).unwrap().starts_with(name))
            .ok_or_else(|| MidiError::DeviceNotFound(name.to_owned()))?;
        let in_name = midi_in
            .port_name(&in_port)
            .expect("failed to query MIDI port name");

        let midi_out = MidiOutput::new(&format!("StageBridge_out_{}", in_name))
            .map_err(|e| MidiError::from(e))?;
        let out_port = midi_out
            .ports()
            .into_iter()
            .find(|p| midi_out.port_name(p).unwrap().starts_with(name))
            .ok_or_else(|| MidiError::DeviceNotFound(name.to_owned()))?;
        let out_name = midi_out
            .port_name(&out_port)
            .expect("failed to query MIDI port name");

        let mut out_conn = midi_out
            .connect(&out_port, "out")
            .map_err(|_| MidiError::ConnectionFailed(out_name.clone()))?;

        // Output sender loop
        let _state = Arc::clone(&state);
        let _out_name = out_name.clone();
        task::spawn(async move {
            loop {
                let output = out_rx
                    .recv()
                    .await
                    .expect(&format!("{}: output receiver dropped", &_out_name));

                let data = _state.lock().process_output(output);
                // log::trace!("{} -> {:X?}", &_out_name, &data);

                match out_conn.send(&data) {
                    Err(_) => log::error!("{}: failed to send output", _out_name),
                    _ => {}
                }
            }
        });

        let _in_tx = in_tx.clone();
        let _in_name = in_name.clone();
        let _state = Arc::clone(&state);

        let input_conn = midi_in
            .connect(
                &in_port,
                "in",
                move |_, data, _| {
                    if let Some(input) = _state.lock().process_input(data) {
                        // log::trace!("{} <- [{:X?}]", &_in_name, data);
                        match _in_tx.send(input) {
                            Err(_) => log::warn!("{}: dropped input", &_in_name),
                            _ => {}
                        };
                    }
                },
                (),
            )
            .map_err(|_| MidiError::ConnectionFailed(in_name))?;

        Ok(Self {
            state,

            out_tx,
            in_tx,

            _input: Arc::new(Mutex::new(input_conn)),
        })
    }

    pub fn state(&self) -> Arc<Mutex<D>> {
        Arc::clone(&self.state)
    }

    pub async fn send(&self, output: D::Output) {
        self.out_tx
            .send(output)
            .await
            .expect("output receiver dropped")
    }

    pub fn spawn<F, Fut>(&self, f: F)
    where
        F: FnOnce(Midi<D>) -> Fut + Send + Sync,
        Fut: Future<Output = ()> + Send + 'static
    {
        task::spawn(f((*self).clone()));
    }

    // pub fn spawn_rx<F, Fut>(&self, f: F)
    // where
    //     F: FnMut(Self, D::Input) -> Fut + Send + Sync,
    //     Fut: Future<Output = ()> + Send + 'static
    // {
    //     self.spawn(async move |midi| {
    //         let mut rx = midi.subscribe();
    //         while let Ok(input) = rx.recv().await {
    //             task::spawn(f(midi.clone(), input));
    //         }
    //     });
    // }
}

use super::util::future::Broadcast;
impl <D: Device> Broadcast<D::Input> for Midi<D> {
    fn subscribe(&self) -> broadcast::Receiver<D::Input> {
        self.in_tx.subscribe()
    }
}

pub trait Device: Send + Debug + 'static {
    type Input: Send + Clone + Debug = Vec<u8>;
    type Output: Send + Clone + Debug = Vec<u8>;

    fn new() -> Self;

    fn process_input(&mut self, data: &[u8]) -> Option<<Self as Device>::Input>;
    fn process_output(&mut self, output: <Self as Device>::Output) -> Vec<u8>;
}
