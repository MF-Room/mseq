mod acid;
mod arp;
mod clock;
mod conductor;
mod midi_connection;
mod midi_controller;
mod note;
mod tests;
mod track;

// Interface
pub use conductor::Conductor;
pub use midi_connection::MidiConnection;
use midi_connection::{MidiError, MidirConnection};
pub use midi_controller::{MidiController, MidiNote};
pub use note::Note;
pub use track::{DeteTrack, Track};

use clock::Clock;
use thiserror::Error;

const DEFAULT_BPM: u8 = 120;

#[derive(Error, Debug)]
pub enum MSeqError {
    #[error("Midi error [{}: {}]", file!(), line!())]
    Midi(#[from] MidiError),
    #[error("{0}")]
    Acid(#[from] acid::AcidError),
}

pub struct Context<T: MidiConnection> {
    pub midi: MidiController<T>,
    pub(crate) clock: Clock,
    step: u32,
    running: bool,
    on_pause: bool,
    pause: bool,
}

impl<T: MidiConnection> Context<T> {
    pub fn set_bpm(&mut self, bpm: u8) {
        self.clock.set_bpm(bpm);
    }
    pub fn quit(&mut self) {
        self.running = false
    }
    pub fn pause(&mut self) {
        self.on_pause = true;
        self.pause = true;
        self.midi.stop();
    }
    pub fn resume(&mut self) {
        self.on_pause = false;
        self.midi.send_continue();
    }
    pub fn start(&mut self) {
        self.step = 0;
        self.on_pause = false;
        self.midi.start();
    }
    pub fn get_step(&mut self) -> u32 {
        self.step
    }
    pub fn run(&mut self, mut conductor: impl Conductor<T>) {
        while self.running {
            conductor.update(self);

            self.clock.tick();
            self.midi.send_clock();

            if !self.on_pause {
                self.step += 1;
                self.midi.update(self.step);
            } else if self.pause {
                self.midi.pause();
                self.pause = false;
            }
        }
        self.midi.stop();
        self.clock.tick();
        self.midi.pause();
    }
}

pub fn run(
    mut conductor: impl Conductor<MidirConnection>,
    port: Option<u32>,
) -> Result<(), MSeqError> {
    let conn = MidirConnection::new(port)?;
    let midi = MidiController::new(conn);

    let mut ctx = Context {
        midi,
        clock: Clock::new(DEFAULT_BPM),
        step: 0,
        running: true,
        on_pause: true,
        pause: false,
    };

    conductor.init(&mut ctx);
    ctx.run(conductor);

    Ok(())
}

pub fn param_value(v: f32) -> u8 {
    if v < -1.0 {
        return 0;
    }
    if v > 1.0 {
        return 127;
    }
    63 + (v * 63.0).round() as u8
}

#[allow(unused)]
macro_rules! log_trace { ($($x:tt)*) => (
    #[cfg(feature = "log")] {
        log::trace!($($x)*)
    }
) }
#[allow(unused)]
macro_rules! log_debug { ($($x:tt)*) => (
    #[cfg(feature = "log")] {
        log::debug!($($x)*)
    }
) }
#[allow(unused)]
macro_rules! log_info { ($($x:tt)*) => (
    #[cfg(feature = "log")] {
        log::info!($($x)*)
    }
) }
#[allow(unused)]
macro_rules! log_warn { ($($x:tt)*) => (
    #[cfg(feature = "log")] {
        log::warn!($($x)*)
    }
) }
#[allow(unused)]
macro_rules! log_error { ($($x:tt)*) => (
    #[cfg(feature = "log")] {
        log::error!($($x)*)
    }
) }
#[allow(unused)]
pub(crate) use {log_debug, log_error, log_info, log_trace, log_warn};
