use alloc::vec::Vec;

use crate::{Context, MidiMessage, midi_controller::Instruction};

/// Output of [`Conductor::handle_input`].
///
/// Carries two independent channels:
/// - `instructions` are handed to the MIDI controller, where notes are buffered and
///   step-scheduled. They are executed only while the sequencer is running and are
///   dropped while paused.
/// - `messages` are forwarded directly to the MIDI output, bypassing the controller's
///   note buffering. They are always sent, including while paused.
#[derive(Default)]
pub struct InputResponse {
    /// Instructions processed by the MIDI controller (buffered / step-scheduled).
    /// Executed only while the sequencer is running; dropped while paused.
    pub instructions: Vec<Instruction>,
    /// MIDI messages forwarded directly to the output, bypassing the controller.
    /// Always sent, including while paused.
    pub messages: Vec<MidiMessage>,
}

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
    /// `update` is called on every tick, but while paused (via [`Context::pause`])
    /// the returned instructions are dropped rather than sent to the MIDI output.
    /// Use [`Context::is_paused`] if you want to alter behavior while paused.
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
    /// The returned [`InputResponse`] carries two channels:
    ///
    /// - `instructions` are passed to the MIDI controller (buffered / step-scheduled).
    ///   They are executed only while the sequencer is running and are dropped while paused.
    /// - `messages` are forwarded directly to the MIDI output, bypassing the controller.
    ///   They are always sent, including while paused.
    ///
    /// Use [`Context::is_paused`] if you want to alter behavior while paused.
    ///
    /// # Parameters
    ///
    /// - `input_id`: 0-based index identifying which MIDI input produced the message. It matches the
    ///   position of the corresponding input in the list of inputs passed to the runtime. When a single
    ///   input is used, this is always `0`.
    /// - `input`: The received [`MidiMessage`].
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
    fn handle_input(
        &mut self,
        _input_id: usize,
        _input: MidiMessage,
        _context: &Context,
    ) -> InputResponse {
        InputResponse::default()
    }
}
