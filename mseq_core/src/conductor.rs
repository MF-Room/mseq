use alloc::vec;
use alloc::vec::Vec;

use crate::{Context, MidiMessage, midi_controller::Instruction};

/// The Conductor trait is the trait that the user has to implement to be able to use mseq. The user
/// has to implement [`Conductor::init`] and [`Conductor::update`], then pass the Conductor to the
/// main entry point [`crate::run`]. Refer to these [`examples`] for complete implementation
/// examples.
///
/// [`examples`]: https://github.com/MF-Room/mseq/tree/main/examples
pub trait Conductor {
    /// This function will be called only once at the start when the Conductor is passed to
    /// [`crate::run`] (the main entry point).
    fn init(&mut self, context: &mut Context);
    /// This function will be called at every midi clock cycle. All the notes sent to the
    /// [`Context::midi`] will be played at the beginning of the next midi clock cycle.
    ///
    /// __Warning: if this function takes too long, the midi clock might be late. Be careful not to
    /// do any intensive computation, or block the thread.__
    fn update(&mut self, context: &mut Context) -> Vec<Instruction>;
    /// Midi input callback function. Default implementation does nothing.
    fn handle_input(&mut self, _input: MidiMessage, _context: &Context) -> Vec<Instruction> {
        vec![]
    }
}
