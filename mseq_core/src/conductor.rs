use crate::{Context, MidiOut};

/// The Conductor trait is the trait that the user has to implement to be able to use mseq. The user
/// has to implement [`Conductor::init`] and [`Conductor::update`], then pass the Conductor to the
/// main entry point [`crate::run`]. Refer to these [`examples`] for complete implementation
/// examples.
///
/// [`examples`]: https://github.com/MF-Room/mseq/tree/main/examples
pub trait Conductor {
    /// This function will be called only once at the start when the Conductor is passed to
    /// [`crate::run`] (the main entry point).
    fn init(&mut self, context: &mut Context<impl MidiOut>);
    /// This function will be called at every midi clock cycle. All the notes sent to the
    /// [`Context::midi`] will be played at the beginning of the next midi clock cycle.
    ///
    /// __Warning: if this function takes too long, the midi clock might be late. Be careful not to
    /// do any intensive computation, ot block the thread.__
    fn update(&mut self, context: &mut Context<impl MidiOut>);
}
