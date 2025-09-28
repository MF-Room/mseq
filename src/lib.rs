//! # mseq
//!
//! `mseq` is a lightweight MIDI sequencer framework written in Rust.  
//! It provides a flexible core for building sequencers that can run in **standalone**, **master**, or
//! **slave** mode, with synchronization over standard MIDI clock and transport messages.
//!
//! ## Overview
//!
//! The sequencer is driven by a user-provided [`Conductor`] implementation, which defines how the
//! sequencer initializes, progresses at each clock tick, and reacts to external MIDI messages.
//! The core engine manages timing, MIDI input/output, and synchronization.
//!
//! - **No input** → runs standalone with its internal clock and transport, generating MIDI clock and
//!   transport messages but ignoring external MIDI input.
//! - **Master mode** → runs with its internal clock while also processing incoming MIDI events
//!   (except for external clock/transport).
//! - **Slave mode** → synchronizes playback to an external MIDI clock and responds to
//!   Start/Stop/Continue messages, dynamically adjusting BPM to match the clock source.
//!
//! ## Usage
//!
//! The entry point of the crate is the [`run`] function:
//!
//! ```no_run
//! use mseq::{run, Conductor, Context, Instruction, MidiInParam, MidiMessage};
//!
//! struct MyConductor;
//!
//! impl Conductor for MyConductor {
//!     fn init(&mut self, _ctx: &mut Context) -> Vec<Instruction> {
//!         // Return setup instructions (e.g., reset all controllers)
//!         vec![]
//!     }
//!
//!     fn update(&mut self, _ctx: &mut Context) -> Vec<Instruction> {
//!         // Called each clock tick: return note events or other MIDI instructions
//!         vec![]
//!     }
//!
//!     fn handle_input(&mut self, input: MidiMessage, _ctx: &Context) -> Vec<Instruction> {
//!         vec![]
//!     }
//! }
//!
//! fn main() -> Result<(), mseq::MSeqError> {
//!     let conductor = MyConductor;
//!     let out_port = None; // Ask user for output port
//!     let midi_in = None;  // Run standalone (no input, master clock/transport)
//!     run(conductor, out_port, midi_in)
//! }
//! ```
//!
//! ## Features
//!
//! - Real-time MIDI clock generation and synchronization
//! - Master/slave transport control with Start/Stop/Continue handling
//! - Flexible [`Conductor`] trait for defining sequencer logic
//! - Easy-to-implement tracks via the [`Track`] trait  
//! - Thread-safe, minimal core designed for real-time responsiveness

#![warn(missing_docs)]

mod clock;
mod midi_connection;

pub use midi_connection::MidiInParam;
pub use midir::Ignore;
pub use mseq_core::*;
pub use mseq_tracks::*;

use clock::Clock;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;
use std::time::Instant;

use thiserror::Error;

use crate::midi_connection::*;

/// Error type of mseq
#[derive(Error, Debug)]
pub enum MSeqError {
    /// Error type related to MIDI messages
    #[error("Midi error [{}: {}]", file!(), line!())]
    Midi(#[from] MidiError),
    /// Error type related to MIDI file parsing
    #[error("Failed to parse midi file [{f}: {l}]\n\t{0}", f=file!(), l=line!())]
    Track(#[from] TrackError),
}

/// `mseq` entry point.  
///
/// This function starts the MIDI sequencer by running the given [`Conductor`] implementation.  
///
/// # Parameters
/// - `conductor`: User-provided implementation of the [`Conductor`] trait, which defines how the
///   sequencer generates and responds to musical events.
/// - `out_port`: MIDI output port ID used to send messages.  
///   If set to `None`, information about available MIDI output ports will be displayed and the user
///   will be prompted to select one.
/// - `midi_in`: Optional [`MidiInParam`] specifying how to configure the MIDI input connection.  
///   If provided, the sequencer can run in either **master mode** (internal clock) or **slave mode**
///   (synchronized to external MIDI clock and transport messages).
///
/// # Behavior
/// - **No input (`midi_in = None`)** → sequencer runs with its internal clock and transport,  
///   generating MIDI clock and transport messages, but ignoring any external MIDI input.  
/// - **Master mode** → sequencer generates its own MIDI clock and transport messages while also
///   handling incoming MIDI events (except for clock/transport).  
/// - **Slave mode** → sequencer synchronizes to external MIDI clock, Start/Stop/Continue messages,
///   and dynamically adjusts BPM to match the external clock source.
///
/// # Errors
/// Returns an [`MSeqError`] if a MIDI port cannot be opened, if MIDI input/output fails, or if track
/// parsing encounters an error.
pub fn run(
    conductor: impl Conductor + std::marker::Send + 'static,
    out_port: Option<u32>,
    midi_in: Option<MidiInParam>,
) -> Result<(), MSeqError> {
    let midi_out = StdMidiOut::new(out_port)?;
    let midi_controller = MidiController::new(midi_out);
    let ctx = Context::default();

    if let Some(params) = midi_in {
        let run = Arc::new(Mutex::new((conductor, midi_controller, ctx)));
        let run_consumer = run.clone();
        let midi_in = connect(params)?;
        let message = midi_in.message.clone();
        thread::spawn(move || {
            loop {
                let r = run_consumer.lock().unwrap();
                let mut r = message.1.wait(r).unwrap();
                let (ref mut conductor, ref mut controller, ref mut ctx) = *r;
                let mut queue = message.0.lock().unwrap();
                ctx.handle_input(conductor, controller, &mut queue);
            }
        });
        if let Some((sys_queue, cond_var)) = midi_in.slave_system {
            run_slave(run, sys_queue, cond_var)
        } else {
            run_master(run)
        }
    } else {
        run_no_input(ctx, midi_controller, conductor)
    }
}

fn run_no_input(
    mut ctx: Context,
    mut controller: MidiController<impl MidiOut>,
    mut conductor: impl Conductor,
) -> Result<(), MSeqError> {
    ctx.init(&mut conductor, &mut controller);
    let mut clock = Clock::new();
    while ctx.is_running() {
        ctx.process_pre_tick(&mut conductor, &mut controller);
        clock.tick(&Duration::from_micros(ctx.get_period_us()));
        ctx.process_post_tick(&mut controller);
    }
    controller.finish();
    // To make sure the midi buffer has time to empty
    clock.tick(&Duration::from_micros(ctx.get_period_us()));
    Ok(())
}

fn run_master(
    run: Arc<Mutex<(impl Conductor, MidiController<impl MidiOut>, Context)>>,
) -> Result<(), MSeqError> {
    {
        let mut r = run.lock().unwrap();
        let (ref mut conductor, ref mut controller, ref mut ctx) = *r;
        ctx.init(conductor, controller);
    }
    let mut clock = Clock::new();

    loop {
        let period_us = {
            let mut r = run.lock().unwrap();
            let (ref mut conductor, ref mut controller, ref mut ctx) = *r;
            ctx.process_pre_tick(conductor, controller);
            ctx.get_period_us()
        };
        clock.tick(&Duration::from_micros(period_us));
        let mut r = run.lock().unwrap();
        let (_, ref mut controller, ref mut ctx) = *r;
        ctx.process_post_tick(controller);
        if !ctx.is_running() {
            break;
        }
    }
    let mut r = run.lock().unwrap();
    let (_, ref mut controller, ref mut ctx) = *r;
    controller.finish();
    // To make sure the midi buffer has time to empty
    clock.tick(&Duration::from_micros(ctx.get_period_us()));
    Ok(())
}

fn run_slave(
    run: Arc<Mutex<(impl Conductor, MidiController<impl MidiOut>, Context)>>,
    sys_queue: Arc<Mutex<InputQueue>>,
    sys_cond_var: Arc<Condvar>,
) -> Result<(), MSeqError> {
    {
        let mut r = run.lock().unwrap();
        let (ref mut conductor, ref mut controller, ref mut ctx) = *r;
        ctx.init(conductor, controller);

        // We start in pause mode
        ctx.pause();
    }

    // We use the average duration over 24 clock messages (1 beat) to set the BPM
    let mut bpm_counter = 0;
    let mut bmp_time_stamp = Instant::now();
    loop {
        {
            let mut r = run.lock().unwrap();
            let (ref mut conductor, ref mut controller, ref mut ctx) = *r;
            ctx.process_pre_tick(conductor, controller);
        }

        // Check the slave system queue
        enum SysMessage {
            Start,
            Stop,
            Continue,
        }
        let mut sys_message = None;

        // We quit the loop if we receive clock message
        loop {
            let mut mutex = sys_queue.lock().unwrap();
            let queue = &mut *mutex;
            let mut quit_loop = false;

            while let Some(message) = queue.pop_front() {
                match message {
                    MidiMessage::Clock => {
                        quit_loop = true;
                    }
                    MidiMessage::Start => {
                        sys_message = Some(SysMessage::Start);
                    }
                    MidiMessage::Stop => {
                        sys_message = Some(SysMessage::Stop);
                    }
                    MidiMessage::Continue => {
                        sys_message = Some(SysMessage::Continue);
                    }
                    _ => unreachable!(),
                }
            }

            if quit_loop {
                break;
            }

            let _r = sys_cond_var.wait(mutex).unwrap();
        }

        let mut r = run.lock().unwrap();
        let (_, ref mut controller, ref mut ctx) = *r;

        bpm_counter += 1;
        if bpm_counter == 24 {
            bpm_counter = 0;
            let duration = bmp_time_stamp.elapsed().as_millis();
            if duration != 0 {
                let bpm = 60000 / duration;
                ctx.set_bpm(bpm as u8);
            }
            bmp_time_stamp = Instant::now();
        }

        if let Some(sys_message) = sys_message {
            match sys_message {
                SysMessage::Start => ctx.start(),
                SysMessage::Stop => ctx.pause(),
                SysMessage::Continue => ctx.resume(),
            }
        }
        ctx.process_post_tick(controller);
        if !ctx.is_running() {
            break;
        }
    }
    let mut r = run.lock().unwrap();
    let (_, ref mut controller, ref mut ctx) = *r;
    controller.finish();
    // To make sure the midi buffer has time to empty
    let mut clock = Clock::new();
    clock.tick(&Duration::from_micros(ctx.get_period_us()));
    Ok(())
}
