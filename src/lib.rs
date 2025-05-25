mod clock;
mod midi_connection;

pub use mseq_core::*;
use mseq_tracks::TrackError;

use std::time::Duration;

use clock::Clock;

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
pub fn run(conductor: impl Conductor, port: Option<u32>) -> Result<(), MSeqError> {
    let midi_out = StdMidiOut::new(port)?;
    let midi_controller = MidiController::new(midi_out);
    let ctx = Context::new();
    run_ctx(ctx, midi_controller, conductor)
}

/// `mseq` entry point when the `MidiOut` connection is already setup. Useful if the user wants to
/// provide their own `MidiOut` implementation.
pub fn run_ctx(
    mut ctx: Context,
    mut controller: MidiController<impl MidiOut>,
    mut conductor: impl Conductor,
) -> Result<(), MSeqError> {
    conductor.init(&mut ctx);
    let mut clock = Clock::new();

    while ctx.is_running() {
        ctx.process_pre_tick(&mut conductor, &mut controller);
        clock.tick(&Duration::from_micros(ctx.get_period_us()));
        ctx.process_post_tick(&mut controller);
    }
    controller.stop_all_notes(None);
    clock.tick(&Duration::from_micros(ctx.get_period_us()));
    controller.stop();

    Ok(())
}
