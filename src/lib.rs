//! TODO

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

/// `mseq` entry point. Run the sequencer by providing a conductor implementation. `port` is the
/// MIDI port id used to send the midi messages. If set to `None`, information about the MIDI ports
/// will be displayed and the output port will be asked to the user with a prompt.
/// TODO update doc
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
        let cond_var = Arc::new(Condvar::new());
        let run_consumer = run.clone();
        let cond_var_consumer = cond_var.clone();
        let midi_in = connect(params.clone(), cond_var)?;
        thread::spawn(move || {
            loop {
                let r = run_consumer.lock().unwrap();
                let mut r = cond_var_consumer.wait(r).unwrap();
                let (ref mut conductor, ref mut controller, ref mut ctx) = *r;
                let mut queue = midi_in.queue.lock().unwrap();
                ctx.handle_input(conductor, controller, &mut *queue);
            }
        });
        if params.slave {
            run_slave(run)
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
    controller.stop_all_notes();
    clock.tick(&Duration::from_micros(ctx.get_period_us()));
    controller.stop();

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
    controller.stop_all_notes();
    clock.tick(&Duration::from_micros(ctx.get_period_us()));
    controller.stop();
    Ok(())
}

fn run_slave(
    run: Arc<Mutex<(impl Conductor, MidiController<impl MidiOut>, Context)>>,
) -> Result<(), MSeqError> {
    todo!();
}
