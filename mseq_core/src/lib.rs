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
#![no_std]

extern crate alloc;

mod bpm;
mod conductor;
mod midi;
mod midi_controller;
mod midi_out;
mod note;
mod track;

use alloc::collections::vec_deque::VecDeque;
// Interface
pub use conductor::Conductor;
pub use midi::*;
pub use midi_controller::*;
pub use midi_out::MidiOut;
pub use note::Note;
pub use track::*;

#[cfg(not(feature = "std"))]
mod no_std_mod {
    extern crate alloc;
    pub use alloc::{string::*, vec, vec::*};
    pub use core::hash;
    pub use core::{convert, fmt};
    pub use hashbrown::{HashMap, HashSet};
}

use bpm::Bpm;

const DEFAULT_BPM: u8 = 120;

/// An object of type [`Context`] is passed to the user [`Conductor`] at each clock tick through the
/// method [`Conductor::update`]. This structure provides the user with a friendly MIDI interface.
/// The user can set some MIDI System Parameters (e.g., [`Context::set_bpm`]) or send some MIDI
/// System Messages (e.g., [`Context::start`]) using directly the [`Context`] methods. The user can
/// also send MIDI Channel Messages (e.g., [`MidiController::play_note`] or
/// [`MidiController::play_track`]) using the field [`Context::midi`].
pub struct Context {
    /// Field used to send MIDI Channel Messages.
    bpm: Bpm,
    step: u32,
    running: bool,
    on_pause: bool,
    pause: bool,
}

/// Inputs queue to process.
pub type InputQueue = VecDeque<
    MidiMessage, // Midi message
>;

impl Context {
    /// Build new mseq context.
    pub fn new() -> Self {
        Self {
            bpm: Bpm::new(DEFAULT_BPM),
            step: 0,
            running: true,
            on_pause: false,
            pause: false,
        }
    }
    /// Set the BPM (Beats per minute) of the sequencer.
    pub fn set_bpm(&mut self, bpm: u8) {
        self.bpm.set_bpm(bpm);
    }

    /// Get the current BPM of the sequencer
    pub fn get_bpm(&self) -> u8 {
        self.bpm.get_bpm()
    }

    /// Get the current period (in microsec) of the sequencer.
    /// A period represents the amount of time between each MIDI clock messages.
    pub fn get_period_us(&self) -> u64 {
        self.bpm.get_period_us()
    }

    /// Stop and exit the sequencer.
    pub fn quit(&mut self) {
        self.running = false
    }

    /// Pause the sequencer and send a MIDI stop message.
    pub fn pause(&mut self) -> Instruction {
        self.on_pause = true;
        self.pause = true;
        Instruction::StopAllNotes { channel_id: None }
    }

    /// Resume the sequencer and send a MIDI continue message.
    pub fn resume(&mut self) -> Instruction {
        self.on_pause = false;
        Instruction::Continue
    }

    /// Start the sequencer and send a MIDI start message. The current step is set to 0.
    pub fn start(&mut self) -> Instruction {
        self.step = 0;
        self.on_pause = false;
        Instruction::Start
    }

    /// Retrieve the current MIDI step.
    /// - 96 steps make a bar
    /// - 24 steps make a whole note
    /// - 12 steps make a half note
    /// - 6 steps make a quarter note
    pub fn get_step(&self) -> u32 {
        self.step
    }

    /// MIDI logic called before the clock tick.
    /// The user doesn't need to call this function.
    pub fn process_pre_tick(
        &mut self,
        conductor: &mut impl Conductor,
        controller: &mut MidiController<impl MidiOut>,
    ) {
        conductor
            .update(self)
            .into_iter()
            .for_each(|instruction| controller.execute(instruction));
    }

    /// MIDI logic called after the clock tick.
    /// The user doesn't need to call this function.
    pub fn process_post_tick(&mut self, controller: &mut MidiController<impl MidiOut>) {
        controller.send_clock();
        if !self.on_pause {
            self.step += 1;
            controller.update(self.step);
        } else if self.pause {
            controller.stop();
            self.pause = false;
        }
    }

    /// Return true if the sequencer is running, false if the sequencer should stop.
    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn handle_inputs(
        &mut self,
        conductor: &mut impl Conductor,
        controller: &mut MidiController<impl MidiOut>,
        input_queue: &mut InputQueue,
    ) {
        input_queue
            .drain(..)
            .flat_map(|message| conductor.handle_input(message, self))
            .for_each(|instruction| controller.execute(instruction));
    }
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
    63 + (v * 63.0) as u8
}
