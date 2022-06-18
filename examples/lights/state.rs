use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use stagebridge::util::future::Broadcast;
use tokio::sync::{oneshot, mpsc, broadcast};
use tokio::task::spawn;

use stagebridge::midi::device::launchpad_x::types::PaletteColor;

#[derive(Clone, Copy, Debug)]
pub struct State {
    pub active: bool,
    pub bpm: f32,
    pub div: (u16, u16),
    pub ofs_light: f32,
    pub ofs_pad: f32,

    pub palette: Palette,

    pub mode: Mode,
    pub block: bool,
    pub pause: bool,
    pub pause_enable: bool,
    pub primary: PaletteColor,
    pub secondary: PaletteColor,
    pub accent: PaletteColor,
    pub alpha: f32,
    pub min: f32,
    pub max: f32,
    pub duty: f32,
}

#[derive(Clone, Copy, Debug)]
pub enum Palette {
    Main,
    Foo,
    Bar,
    Color,
}

#[derive(Clone, Copy, Debug)]
pub enum Mode {
    Off,
    Manual,

    Solid, // solid c1
    SolidHue,
    SolidHueRotate,
    Solid2, // solid c1/c2
    Solid4, // solid c1/c2/c3/c4

    Bounce, // c1 bounces from full to off over 1 beat
    BounceHue, // hue bounces from full to off over 1 beat, hue rotates over 8 beats
    BounceFlash, // c1 bounces, accent div with c2 flash

    SawUp,
    SawDown,
    SawUpHue,
    SawDownHue,

    Strobe, // stroob
    StrobeAccent, // stroob, but every div/4 is c2

    Chase, // a single light rotates around, all the rest off
    ChaseHalf, // two lights (left and right) bounce between front and back
    ChaseRotate, // two lights rotate

    Alternate, // switch even/odd primary/secondary
    AlternateFlash, // switch even/odd primary/secondary, accent div with solid flash
}

#[derive(Debug)]
pub enum Command {
    Get { tx: oneshot::Sender<State> },

    Active(bool),
    Bpm(f32),
    Div((u16, u16)),
    OffsetLight(f32),
    OffsetPad(f32),

    Palette(Palette),

    Mode(Mode),
    Block(bool),
    Pause(bool),
    PauseEnable(bool),
    Refresh,
    PrimaryColor(PaletteColor),
    SecondaryColor(PaletteColor),
    AccentColor(PaletteColor),
    Alpha(f32),
    Min(f32),
    Max(f32),
    Duty(f32),
}

pub struct StateHolder {
    cmd_tx: mpsc::Sender<Command>,
    state_tx: broadcast::Sender<State>,
    mode_tx: broadcast::Sender<State>,

    // TODO: UGLY
    pause: Arc<AtomicBool>,
}

impl StateHolder {
    pub fn spawn() -> Self {
        let (cmd_tx, mut cmd_rx) = mpsc::channel(16);
        let (state_tx, _) = broadcast::channel(16);
        let (mode_tx, _) = broadcast::channel(16);

        let pause = Arc::new(AtomicBool::new(false));

        let _state_tx = state_tx.clone();
        let _mode_tx = mode_tx.clone();
        let _pause = Arc::clone(&pause);
        spawn(async move {
            let mut state = State {
                active: true,
                bpm: 120.0,
                div: (1, 4),
                ofs_light: 0.0,
                ofs_pad: 0.0,

                palette: Palette::Main,

                mode: Mode::Off,
                block: false,
                pause: false,
                pause_enable: false,
                primary: PaletteColor::Cyan,
                secondary: PaletteColor::Magenta,
                accent: PaletteColor::White,
                alpha: 1.0,
                min: 0.0,
                max: 1.0,
                duty: 0.2,
            };

            while let Some(cmd) = cmd_rx.recv().await {
                log::trace!("{:?}", cmd);
                
                match cmd {
                    Command::Get { tx } => { 
                        match tx.send(state) { _ => {} }; 
                        continue;
                    },
                    Command::Active(active) => {
                        state.active = active;
                        log::debug!("Active -> {}", active);
                        // TODO: UGLY
                        match _state_tx.send(state) { _ => {} }
                        continue;
                    }
                    Command::Bpm(bpm) => {
                        state.bpm = bpm;
                        log::debug!("BPM -> {}", bpm);
                        // TODO: UGLY
                        match _state_tx.send(state) { _ => {} }
                        continue;
                    }
                    _ => {}
                }

                if !state.active { continue }

                match cmd {
                    Command::Palette(p) => {
                        state.palette = p;
                        log::debug!("Palette -> {:?}", p);
                        // TODO: UGLY
                        match _state_tx.send(state) { _ => {} }
                        continue;
                    }
                    Command::Block(b) => {
                        state.block = b;
                        log::debug!("Block -> {}", b);
                        // TODO: UGLY
                        match _state_tx.send(state) { _ => {} }
                        continue;
                    },
                    Command::Pause(p) => {
                        state.pause = p;
                        log::debug!("Pause -> {}", p);
                        _pause.store(p, Ordering::Release);
                        // TODO: UGLY
                        match _state_tx.send(state) { _ => {} }
                        continue;
                    },
                    Command::PauseEnable(p) => {
                        state.pause_enable = p;
                        log::debug!("Pause Enable -> {}", p);
                        // TODO: UGLY
                        match _state_tx.send(state) { _ => {} }
                        continue;
                    },
                    _ => {},
                }

                match cmd {
                    Command::Div((num, denom)) => {
                        state.div = (num, denom);
                        log::debug!("Div -> {}/{}", num, denom);
                    }
                    Command::OffsetLight(dt) => {
                        state.ofs_light = dt;
                        log::debug!("Light Offset -> {}", dt);
                    }
                    Command::OffsetPad(dt) => {
                        state.ofs_pad = dt;
                        log::debug!("Pad Offset -> {}", dt);
                    }

                    Command::Mode(m) => {
                        state.mode = m;
                        log::debug!("Mode -> {:?}", m);
                    }
                    Command::Refresh => {
                        log::debug!("Refresh");
                    },
                    Command::PrimaryColor(c) => {
                        state.primary = c;
                        log::debug!("Primary -> {:?}", c);
                    }
                    Command::SecondaryColor(c) => {
                        state.secondary = c;
                        log::debug!("Secondary -> {:?}", c);
                    }
                    Command::AccentColor(c) => {
                        state.accent = c;
                        log::debug!("Accent -> {:?}", c);
                    }
                    Command::Alpha(a) => {
                        state.alpha = a;
                        log::debug!("Light Alpha -> {:?}", a);
                    }
                    Command::Min(fr) => {
                        state.min = fr;
                        log::debug!("Min -> {:?}", fr);
                    },
                    Command::Max(fr) => {
                        state.max = fr;
                        log::debug!("Max -> {:?}", fr);
                    },
                    Command::Duty(fr) => {
                        state.duty = fr;
                        log::debug!("Duty -> {:?}", fr);
                    },

                    Command::Get {..} => {}
                    Command::Active(_) => {}
                    Command::Bpm(_) => {}
                    Command::Palette(_) => {},
                    Command::Block(_) => {},
                    Command::Pause(_) => {},
                    Command::PauseEnable(_) => {},
                }

                // use broadcast::error::SendError;

                // TODO: UGLY
                match _state_tx.send(state) { _ => {} }
                match _mode_tx.send(state) { _ => {} }
            }

            panic!("state manager exited");
        });

        Self {
            cmd_tx,
            state_tx,
            mode_tx,
            pause
        }
    }

    pub async fn send(&self, cmd: Command) {
        // use tokio::sync::broadcast::error::SendError;
        self.cmd_tx.send(cmd).await.unwrap();
    }

    pub async fn get(&self) -> State {
        let (tx, rx) = oneshot::channel();
        self.send(Command::Get { tx }).await;
        rx.await.unwrap()
    }

    pub fn paused(&self) -> bool {
        self.pause.load(Ordering::Acquire)
    }

    pub fn subscribe_mode(&self) -> broadcast::Receiver<State> {
        self.mode_tx.subscribe()
    }
}

impl Broadcast<State> for StateHolder {
    fn subscribe(&self) -> broadcast::Receiver<State> {
        self.state_tx.subscribe()
    }
}