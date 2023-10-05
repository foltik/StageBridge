use std::thread;
use std::thread::JoinHandle;
use std::sync::mpsc;
use std::time::Duration;
use eyre::ContextCompat;
use midir::MidiInputConnection;
use eyre::Result;

use midir::{MidiInput, MidiOutput};

pub mod device;
use device::Device;

const NAME: &str = "stagebridge";

pub struct Midi<D: Device> {
    in_rx: mpsc::Receiver<D::Input>,
    out_tx: mpsc::Sender<D::Output>,

    _raw: MidiRaw,
    _thread: JoinHandle<()>,
}

impl<D: Device> Midi<D> {
    pub fn connect(name: &str, mut device: D) -> Result<Self> {
        let (in_tx, in_rx) = mpsc::channel::<D::Input>();
        let (out_tx, out_rx) = mpsc::channel::<D::Output>();

        let (_raw, raw_rx, raw_tx) = MidiRaw::connect(name)?;

        let _name = name.to_string();
        let _thread = thread::spawn(move || loop {
            if let Ok(data) = raw_rx.try_recv() {
                if let Some(input) = device.process_input(&data) {
                    log::debug!("{_name} <- {input:?}");
                    in_tx.send(input).unwrap();
                }
            }

            if let Ok(output) = out_rx.try_recv() {
                log::debug!("{_name} -> {output:?}");
                let data = device.process_output(output);
                raw_tx.send(data).unwrap();
            }

            thread::sleep(Duration::from_millis(1));
        });

        Ok(Self { in_rx, out_tx, _raw, _thread })
    }

    pub fn send(&mut self, output: D::Output) {
        self.out_tx.send(output).unwrap();
    }

    pub fn recv(&mut self) -> Vec<D::Input> {
        let mut vec = Vec::default();
        while let Ok(input) = self.in_rx.try_recv() {
            vec.push(input);
        }
        vec
    }

    pub fn list() -> Result<()> {
        let midi_in = MidiInput::new(&format!("{NAME}_list_in"))?;
        let midi_out = MidiOutput::new(&format!("{NAME}_list_out"))?;

        for port in midi_in.ports() {
            log::trace!("IN: '{:?}'", midi_in.port_name(&port).unwrap());
        }

        for port in midi_out.ports() {
            log::trace!("OUT: '{:?}'", midi_out.port_name(&port).unwrap());
        }

        Ok(())
    }
}

pub struct MidiRaw {
    _in_conn: MidiInputConnection<()>,
    _out_thread: JoinHandle<()>,
}

#[allow(clippy::type_complexity)]
impl MidiRaw {
    pub fn connect(name: &str) -> Result<(Self, mpsc::Receiver<Vec<u8>>, mpsc::Sender<Vec<u8>>)> {
        let (in_tx, in_rx) = mpsc::channel::<Vec<u8>>();
        let (out_tx, out_rx) = mpsc::channel::<Vec<u8>>();

        let midi_in = MidiInput::new(&format!("StageBridge_in_{}", name))?;
        let midi_out = MidiOutput::new(&format!("StageBridge_out_{}", name))?;

        let in_port = midi_in.ports().into_iter()
            .find(|p| midi_in.port_name(p).unwrap().starts_with(name))
            .with_context(|| format!("no midi input '{name}'"))?;
        let out_port = midi_out.ports().into_iter()
            .find(|p| midi_out.port_name(p).unwrap().starts_with(name))
            .with_context(|| format!("no midi output '{name}'"))?;

        let _name = name.to_string();
        let _in_conn = midi_in.connect(&in_port, "in", move |_, data, _| {
            log::trace!("{_name} <- [{data:X?}]");
            in_tx.send(data.to_vec()).unwrap();
        }, ()).unwrap();

        let mut out_conn = midi_out.connect(&out_port, "out").unwrap();

        let _name = name.to_string();
        let _out_thread = thread::spawn(move || {
            loop {
                let data = out_rx.recv().unwrap();
                log::trace!("{_name} -> {data:X?}");
                out_conn.send(&data).unwrap();
            }
        });

        Ok((Self { _in_conn, _out_thread }, in_rx, out_tx))
    }
}
