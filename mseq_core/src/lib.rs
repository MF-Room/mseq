//! Core framework for building custom MIDI sequencers.
//!
//! `mseq_core` provides the foundational traits and utilities needed to implement
//! your own MIDI sequencer, with a focus on portability and modularity.
//!
//! This crate is built with `#![no_std]`, making it suitable for embedded platforms
//! as well as standard operating systems.
//!
//! ## Getting Started
//!
//! To create a custom sequencer, you typically:
//!
//! - Implement the [`Conductor`] trait to define your sequencer's control logic.
//! - Define one or more tracks by either:
//!   - Implementing the [`Track`] trait for custom behavior.
//!   - Instantiating [`DeteTrack`] for deterministic, looping patterns.
//!
//! ## Platform Support
//!
//! - For OS-based systems, use the [`mseq`](https://crates.io/crates/mseq) crate — a reference implementation of `mseq_core` for standard platforms.
//! - For embedded development (e.g., STM32F4), see the [`mseq_embedded`](https://github.com/MF-Room/mseq_embedded) repository, which provides an STM32-specific integration of `mseq_core`.
//!
//! ## Crate Features
//!
//! - No `std` dependency (`#![no_std]` compatible).
//! - Modular and extensible design.
//! - Reusable across multiple platforms.

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
use bpm::Bpm;
pub use conductor::Conductor;
pub use midi::*;
pub use midi_controller::*;
pub use midi_out::MidiOut;
pub use note::Note;
pub use track::*;

const DEFAULT_BPM: u8 = 120;

/// An object of type [`Context`] is passed to the user’s [`Conductor`] at each clock tick
/// via the [`Conductor::update`] method. It provides a high-level interface to send
/// system MIDI messages and modify system parameters.
///
/// The user can set MIDI system parameters (e.g., [`Context::set_bpm`]) or send system messages
/// (e.g., [`Context::start`]) using the provided methods.
///
/// In addition to sending the corresponding MIDI system messages, these methods also update
/// the internal logic of the sequencer to reflect the change.
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

impl Default for Context {
    fn default() -> Self {
        Self {
            bpm: Bpm::new(DEFAULT_BPM),
            step: 0,
            running: true,
            on_pause: false,
            pause: false,
        }
    }
}

impl Context {
    /// Sets the BPM (Beats per minute) of the sequencer.
    pub fn set_bpm(&mut self, bpm: u8) {
        self.bpm.set_bpm(bpm);
    }

    /// Gets the current BPM of the sequencer.
    pub fn get_bpm(&self) -> u8 {
        self.bpm.get_bpm()
    }

    /// Gets the current period (in microsec) of the sequencer.
    /// A period represents the amount of time between each MIDI clock messages.
    pub fn get_period_us(&self) -> u64 {
        self.bpm.get_period_us()
    }

    /// Stops and exit the sequencer.
    pub fn quit(&mut self) {
        self.running = false
    }

    /// Pauses the sequencer and send a MIDI stop message.
    pub fn pause(&mut self) -> Instruction {
        self.on_pause = true;
        self.pause = true;
        Instruction::StopAllNotes
    }

    /// Resumes the sequencer and send a MIDI continue message.
    pub fn resume(&mut self) -> Instruction {
        self.on_pause = false;
        Instruction::Continue
    }

    /// Starts the sequencer and send a MIDI start message. The current step is set to 0.
    pub fn start(&mut self) -> Instruction {
        self.step = 0;
        self.on_pause = false;
        Instruction::Start
    }

    /// Retrieves the current MIDI step.
    /// - 96 steps make a bar
    /// - 24 steps make a whole note
    /// - 12 steps make a half note
    /// - 6 steps make a quarter note
    pub fn get_step(&self) -> u32 {
        self.step
    }

    /// MIDI logic called before the clock tick.
    /// This function is not intended to be called directly by users.  
    /// `handle_inputs` is used internally to enable code reuse across platforms.
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
    /// This function is not intended to be called directly by users.  
    /// `handle_inputs` is used internally to enable code reuse across platforms.
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

    /// Returns `true` if the sequencer is currently running, `false` otherwise.
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Internal MIDI input handler.
    ///
    /// This function is not intended to be called directly by users.  
    /// Instead, users should implement [`Conductor::handle_input`] for their custom input handler logic.
    ///
    /// `handle_input` is used internally to enable code reuse across platforms and unify MIDI input processing.
    pub fn handle_input(
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

/// Performs a linear conversion from `[0.0, 1.0]` to [0, 127]. If `v` is smaller than `0.0` return
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
