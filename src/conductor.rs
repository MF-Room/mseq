use crate::{Context, MidiConnection};

/// The Conductor trait is the trait that the client has to implement to be able to use mseq. The
/// client has to implement init and update, then pass the Conductor to the main entry point
/// (`mseq::run`). Refer to the examples in `examples/` for complete examples.
pub trait Conductor {
    /// This function will be called only once when the Conductor is passed to the main entry point.
    fn init(&mut self, context: &mut Context<impl MidiConnection>);
    /// This function will be called at every midi clock. All the notes sent to the MidiController
    /// (`context.midi_controller`) will be played at the next midi clock.
    /// 
    /// __Warning: if this function takes too long, the midi clock might be late. Be careful not to
    /// do any intensive computation, call sleep(), or wait for a lock inside this function.__
    fn update(&mut self, context: &mut Context<impl MidiConnection>);
}
