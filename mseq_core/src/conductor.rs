use alloc::vec;
use alloc::vec::Vec;

use crate::{Context, MidiMessage, midi_controller::Instruction};

/// Entry point for user-defined sequencer behavior.
///
/// The `Conductor` trait must be implemented by the user to define how their sequencer
/// is initialized, updated, and how it responds to external inputs. It serves as the
/// core integration point between the `mseq_core` engine and user-defined sequencing logic.
///
/// This trait provides four methods:
///
/// - [`Self::init`] — Called once at startup to set up state.
/// - [`Self::update`] — Called at each step to update the sequencer logic (e.g., advance position, trigger tracks).
/// - [`Self::handle_input`] — Called when an external input (e.g., MIDI event) is received.
/// - [`Self::context_changed`] — Called when the runtime changes the [`Context`] between two
///   steps (e.g., the external clock stops in slave mode).
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
    /// A [`Context::start`], [`Context::resume`] or [`Context::pause`] called from `update`
    /// applies to the tick being prepared: after `start`, [`Context::get_step`] returns 0 and the
    /// returned instructions play on the next clock, right after the MIDI Start; after `pause`,
    /// they are dropped and the MIDI Stop goes out before that clock.
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
    /// A `Vec<Instruction>` processed by the MIDI controller. [`Instruction::MidiMessage`]
    /// instructions are forwarded to the MIDI output even while the sequencer is paused;
    /// all other instructions are executed only while the sequencer is running and are
    /// dropped while paused.
    fn handle_input(
        &mut self,
        _input_id: usize,
        _input: MidiMessage,
        _context: &Context,
    ) -> Vec<Instruction> {
        vec![]
    }

    /// Called when the runtime changes the [`Context`] between two steps, so that no
    /// [`update`](Self::update) sees the change when it happens.
    ///
    /// In slave mode, `update` only runs on a pulse of the external clock, so while that clock
    /// is stopped nothing else would tell the conductor that the transport or the BPM changed.
    /// Use this method to refresh whatever depends on them, such as a display of
    /// [`Context::get_bpm`] or [`Context::is_paused`].
    ///
    /// The `mseq` runtime calls it in slave mode only, as the context otherwise changes through
    /// the conductor's own calls:
    ///
    /// - right after [`init`](Self::init), once the runtime has paused the sequencer and cleared
    ///   the BPM (a BPM set in `init` is not the external clock's);
    /// - after it applies a Start, Stop or Continue message from the external clock source;
    /// - when the external clock stops (500 ms without a clock pulse) and the BPM becomes
    ///   unknown.
    ///
    /// Does nothing by default.
    fn context_changed(&mut self, _context: &Context) {}
}
