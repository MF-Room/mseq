use alloc::vec;
use alloc::vec::Vec;

use crate::{Context, MidiMessage, midi_controller::Instruction};

/// Entry point for user-defined sequencer behavior.
///
/// The `Conductor` trait must be implemented by the user to define how their sequencer
/// is initialized, updated, and how it responds to external inputs. It serves as the
/// core integration point between the `mseq_core` engine and user-defined sequencing logic.
///
/// This trait provides three key methods:
///
/// - [`Self::update`] — Called at each step to update the sequencer logic (e.g., advance position, trigger tracks).
/// - [`Self::handle_input`] — Called when an external input (e.g., MIDI event) is received.
///
/// # Example
///
/// See example implementations in the [mseq GitHub repository](https://github.com/MF-Room/mseq/tree/main/examples).
pub trait Conductor {
    ///Called once at startup to initialize state.
    ///Returns the set of instructions that should be executed at initialization.
    fn init(&mut self, context: &mut Context) -> Vec<Instruction>;
    /// Called at every clock tick to advance the sequencer state.
    ///
    /// This method is responsible for progressing the sequencer and producing
    /// the set of instructions that should be executed at the current tick (e.g., sending MIDI events).
    ///
    /// # Returns
    ///
    /// A `Vec<Instruction>` containing the actions to be passed to the MIDI controller
    /// or output backend for this tick.
    fn update(&mut self, context: &mut Context) -> Vec<Instruction>;

    /// Handles a single input message and updates the conductor state accordingly.
    ///
    /// This method is called whenever a new [`MidiMessage`] is received.
    /// It allows the conductor to react to external inputs by updating internal state or triggering events.
    ///
    /// The returned `Vec<Instruction>` is passed directly to the MIDI controller or output backend,
    /// allowing the conductor to immediately produce output in response to the input.
    ///
    /// # Intercepted Messages
    ///
    /// Depending on the platform-specific implementation, certain MIDI messages such as
    /// [`MidiMessage::Clock`] may be intercepted by the runtime before reaching this function.
    /// These are typically used for transport control and synchronization.
    ///
    /// # Returns
    ///
    /// A `Vec<Instruction>` to be sent to the MIDI output immediately.
    fn handle_input(&mut self, _input: MidiMessage, _context: &Context) -> Vec<Instruction> {
        vec![]
    }
}
