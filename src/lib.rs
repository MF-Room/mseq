//! Library for developing MIDI Sequencers.
//!
//! To start using `mseq`, create a struct that implements the [`Conductor`] trait.
//!
//! You can then add tracks to your sequencer by adding fields (to your struct that implements the
//! [`Conductor`] trait) of type [`DeteTrack`] or more generally fields that implement the trait
//! [`Track`].
//!
//! Once this is done, you can play your track in the [`Conductor::update`] function of your struct
//! that implements the [`Conductor`] trait. To do so, call the method
//! [`MidiController::play_track`] (of the [`Context::midi`]) with the track you want to play as a
//! parameter.
//!
//! You can find some examples in the [`examples`] directory.
//!
//! [`examples`]: https://github.com/MF-Room/mseq/tree/main/examples

#![warn(missing_docs)]
#![cfg_attr(feature = "embedded", no_std)]

mod acid;
mod arp;
mod clock;
mod conductor;
mod div;
mod midi_connection;
mod midi_controller;
mod note;
mod tests;
mod track;

// Interface
pub use acid::{AcidTrig, Timing};
pub use arp::ArpDiv;
pub use conductor::Conductor;
pub use div::ClockDiv;
pub use midi_connection::MidiConnection;
use midi_connection::{MidiError, MidirConnection};
pub use midi_controller::{MidiController, MidiNote};
pub use note::Note;
pub use track::{DeteTrack, Track};

use clock::Clock;
use midir::{ConnectError, InitError, MidiOutput};

#[cfg(not(feature = "embedded"))]
use promptly::{prompt_default, ReadlineError};
use thiserror::Error;

const DEFAULT_BPM: u8 = 120;

/// Error type of mseq
#[derive(Error, Debug)]
pub enum MSeqError {
    /// Error type related to MIDI messages
    #[error("Midi error [{}: {}]", file!(), line!())]
    Midi(#[from] MidiError),
    /// Error type related to CSV file parsing
    #[error("Failed to parse csv file [{}: {}]\n\t{0}", file!(), line!())]
    Reading(#[from] csv::Error),
    /// Error type related to MIDI file parsing
    #[error("Failed to parse midi file [{}: {}]\n\t{0}", file!(), line!())]
    Track(#[from] track::TrackError),
}

/// An object of type [`Context`] is passed to the user [`Conductor`] at each clock tick through the
/// method [`Conductor::update`]. This structure provides the user with a friendly MIDI interface.
/// The user can set some MIDI System Parameters (e.g., [`Context::set_bpm`]) or send some MIDI
/// System Messages (e.g., [`Context::start`]) using directly the [`Context`] methods. The user can
/// also send MIDI Channel Messages (e.g., [`MidiController::play_note`] or
/// [`MidiController::play_track`]) using the field [`Context::midi`].
pub struct Context<T: MidiConnection> {
    /// Field used to send MIDI Channel Messages.
    pub midi: MidiController<T>,
    pub(crate) clock: Clock,
    step: u32,
    running: bool,
    on_pause: bool,
    pause: bool,
}

impl<T: MidiConnection> Context<T> {
    /// Set the BPM (Beats per minute) of the sequencer.
    pub fn set_bpm(&mut self, bpm: u8) {
        self.clock.set_bpm(bpm);
    }

    /// Stop and exit the sequencer.
    pub fn quit(&mut self) {
        self.running = false
    }

    /// Pause the sequencer and send a MIDI stop message.
    pub fn pause(&mut self) {
        self.on_pause = true;
        self.pause = true;
        self.midi.stop_all_notes();
    }

    /// Resume the sequencer and send a MIDI continue message.
    pub fn resume(&mut self) {
        self.on_pause = false;
        self.midi.send_continue();
    }

    /// Start the sequencer and send a MIDI start message. The current step is set to 0.
    pub fn start(&mut self) {
        self.step = 0;
        self.on_pause = false;
        self.midi.start();
    }

    /// Retrieve the current MIDI step.
    /// - 96 steps make a bar
    /// - 24 steps make a whole note
    /// - 12 steps make a half note
    /// - 6 steps make a quarter note
    pub fn get_step(&mut self) -> u32 {
        self.step
    }

    fn run(&mut self, mut conductor: impl Conductor) {
        while self.running {
            conductor.update(self);

            self.clock.tick();
            self.midi.send_clock();

            if !self.on_pause {
                self.step += 1;
                self.midi.update(self.step);
            } else if self.pause {
                self.midi.stop();
                self.pause = false;
            }
        }
        self.midi.stop_all_notes();
        self.clock.tick();
        self.midi.stop();
    }
}

/// `mseq` entry point. Run the sequencer by providing a conductor implementation. `port` is the
/// MIDI port id used to send the midi messages. If set to `None`, information about the MIDI ports
/// will be displayed and the output port will be asked to the user with a prompt.
#[cfg(not(feature = "embedded"))]
pub fn run(mut conductor: impl Conductor, port: Option<u32>) -> Result<(), MSeqError> {
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

/// Perform a linear conversion from `[0.0, 1.0]` to [0, 127]. If `v` is smaller than `0.0` return
/// 0. If `v` is greater than `1.0` return 127. The main purpose of this function is to be used with
/// MIDI control changes (CC).
pub fn param_value(v: f32) -> u8 {
    if v < -1.0 {
        return 0;
    }
    if v > 1.0 {
        return 127;
    }
    63 + (v * 63.0).round() as u8
}
