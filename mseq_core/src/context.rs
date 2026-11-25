use crate::Conductor;
use crate::Instruction;
use crate::MidiController;
use crate::MidiMessage;
use crate::MidiOut;
use crate::bpm::Bpm;
use alloc::collections::vec_deque::VecDeque;
use alloc::{vec, vec::Vec};

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
    sys_instructions: Vec<Instruction>,
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
            on_pause: true,
            sys_instructions: vec![],
        }
    }
}

impl Context {
    /// Sets the BPM (Beats per minute) of the sequencer.
    pub fn set_bpm(&mut self, bpm: u8) {
        self.bpm.set_bpm(bpm);
    }

    /// Gets the current BPM of the sequencer, or `None` if it is unknown.
    ///
    /// Without a slave input, the BPM is always known: it is the one set with
    /// [`set_bpm`](Self::set_bpm), 120 by default.
    ///
    /// In slave mode, the BPM is measured from the external MIDI clock, once per beat
    /// (24 clocks). It is `None` until the first beat has been measured, and again once the
    /// external clock stops (the desktop `mseq` crate treats 500 ms without a clock pulse
    /// as a stop).
    /// [`Conductor::update`] is only called on a clock pulse, so the runtime calls
    /// [`Conductor::context_changed`] when it clears the BPM: while the clock is stopped,
    /// `update` does not run to see it.
    pub fn get_bpm(&self) -> Option<u8> {
        self.bpm.get_bpm()
    }

    /// Marks the BPM as unknown: [`get_bpm`](Self::get_bpm) returns `None` until the next
    /// [`set_bpm`](Self::set_bpm). The slave loop calls it before the first beat of external
    /// clock has been measured and when the external clock stops.
    /// This function is not intended to be called directly by users.
    pub fn clear_bpm(&mut self) {
        self.bpm.clear();
    }

    /// Gets the current period (in microsec) of the sequencer.
    /// A period represents the amount of time between each MIDI clock messages.
    /// While the BPM is unknown, this is the period of the last known BPM.
    pub fn get_period_us(&self) -> u64 {
        self.bpm.get_period_us()
    }

    /// Stops and exit the sequencer.
    pub fn quit(&mut self) {
        self.running = false
    }

    /// Pauses the sequencer and send a MIDI stop message.
    ///
    /// While paused, the step counter stops advancing, so step-driven tracks hold
    /// their position. [`Conductor::update`] is still called every tick but its
    /// returned instructions are dropped, and the instructions returned by
    /// [`Conductor::handle_input`] are dropped too, except for
    /// [`Instruction::MidiMessage`] instructions which are still forwarded directly
    /// to the MIDI output.
    pub fn pause(&mut self) {
        self.on_pause = true;
        self.sys_instructions.push(Instruction::StopAllNotes);
        self.sys_instructions.push(Instruction::Stop);
    }

    /// Resumes the sequencer and send a MIDI continue message.
    pub fn resume(&mut self) {
        self.on_pause = false;
        self.sys_instructions.push(Instruction::Continue);
    }

    /// Starts the sequencer and send a MIDI start message. The current step is set to 0.
    ///
    /// If it was already running, the notes playing keep playing to their end.
    pub fn start(&mut self) {
        self.step = 0;
        self.on_pause = false;
        self.sys_instructions.push(Instruction::Start);
    }

    /// Retrieves the current MIDI step: in [`Conductor::update`], the step that the next clock
    /// plays. There is one step per MIDI clock:
    /// - 96 steps make a 4/4 bar
    /// - 24 steps make a quarter note (a beat)
    /// - 12 steps make an eighth note
    /// - 6 steps make a sixteenth note
    pub fn get_step(&self) -> u32 {
        self.step
    }

    /// MIDI logic called at the initialization, when the sequencer makes its own clock.
    /// This function is not intended to be called directly by users.
    /// `init` is used internally to enable code reuse across platforms.
    pub fn init(
        &mut self,
        conductor: &mut impl Conductor,
        controller: &mut MidiController<impl MidiOut>,
    ) {
        conductor
            .init(self)
            .into_iter()
            .for_each(|instruction| controller.execute(instruction));
    }

    /// MIDI logic called at the initialization in slave mode, when an external clock source drives
    /// the sequencer through [`process_transport`](Self::process_transport).
    ///
    /// The clock source owns the transport: the sequencer waits paused for its Start, at step 0,
    /// and the transport that [`Conductor::init`] asked for is dropped, not sent. A BPM set in
    /// `init` is not the clock source's: it is unknown until a beat of clock is measured. The
    /// conductor is then told through [`Conductor::context_changed`].
    /// This function is not intended to be called directly by users.
    pub fn init_slave(
        &mut self,
        conductor: &mut impl Conductor,
        controller: &mut MidiController<impl MidiOut>,
    ) {
        self.init(conductor, controller);
        self.sys_instructions.clear();
        self.step = 0;
        self.on_pause = true;
        self.clear_bpm();
        conductor.context_changed(self);
    }

    /// MIDI logic called before the clock tick, when the sequencer makes its own clock: prepares
    /// what the next clock plays.
    /// This function is not intended to be called directly by users.
    /// `process_pre_tick` is used internally to enable code reuse across platforms.
    pub fn process_pre_tick(
        &mut self,
        conductor: &mut impl Conductor,
        controller: &mut MidiController<impl MidiOut>,
    ) {
        self.prepare(conductor, controller);
    }

    /// MIDI logic called after the clock tick, when the sequencer makes its own clock: sends the
    /// clock and what [`process_pre_tick`](Self::process_pre_tick) prepared for it.
    /// This function is not intended to be called directly by users.
    /// `process_post_tick` is used internally to enable code reuse across platforms.
    pub fn process_post_tick(&mut self, controller: &mut MidiController<impl MidiOut>) {
        controller.send_clock();
        self.advance(controller);
    }

    /// MIDI logic called in slave mode, on each transport message of the clock source (see
    /// [`MidiMessage::is_transport`]), in the order they arrive. Other messages are ignored.
    ///
    /// - A Start, Stop or Continue is applied and sent on at once: a master may stop its clock
    ///   across a transport change. As no [`Conductor::update`] may follow, the conductor is told
    ///   through [`Conductor::context_changed`].
    /// - A Clock is sent on, then plays a step: [`Conductor::update`] runs on it, after the
    ///   transport received before it. The first Clock after a Start plays step 0, and the first
    ///   Clock after a Continue plays the step that the Stop left off at.
    ///
    /// The runtime sets the BPM it measures before handing over the Clock, for `update` to see it.
    /// This function is not intended to be called directly by users.
    pub fn process_transport(
        &mut self,
        conductor: &mut impl Conductor,
        controller: &mut MidiController<impl MidiOut>,
        message: MidiMessage,
    ) {
        match message {
            MidiMessage::Clock => {
                controller.send_clock();
                self.prepare(conductor, controller);
                self.advance(controller);
                return;
            }
            MidiMessage::Start => self.start(),
            MidiMessage::Stop => self.pause(),
            MidiMessage::Continue => self.resume(),
            _ => return,
        }
        self.send_sys_instructions(controller);
        conductor.context_changed(self);
    }

    /// Immediately sends the pending system instructions (Start / Stop / Continue /
    /// StopAllNotes) queued by [`start`](Self::start), [`pause`](Self::pause) and
    /// [`resume`](Self::resume).
    /// This function is not intended to be called directly by users.
    #[deprecated(
        since = "2.2.0",
        note = "slave runtimes hand the transport to `process_transport`, which sends it"
    )]
    pub fn flush_sys_instructions(&mut self, controller: &mut MidiController<impl MidiOut>) {
        self.send_sys_instructions(controller);
    }

    fn send_sys_instructions(&mut self, controller: &mut MidiController<impl MidiOut>) {
        core::mem::take(&mut self.sys_instructions)
            .into_iter()
            .for_each(|instruction| controller.execute(instruction));
    }

    /// Prepares what the next clock plays: runs [`Conductor::update`], sends the transport it (or
    /// `init`) asked for, then hands its instructions to the controller unless the sequencer is
    /// paused. The pause is checked after `update`, so that a `start`, `resume` or `pause` it calls
    /// applies to this step.
    fn prepare(
        &mut self,
        conductor: &mut impl Conductor,
        controller: &mut MidiController<impl MidiOut>,
    ) {
        let instructions = conductor.update(self);
        self.send_sys_instructions(controller);
        if !self.on_pause {
            instructions
                .into_iter()
                .for_each(|instruction| controller.execute(instruction));
        }
    }

    /// Moves on to the next step once a clock is sent, unless the sequencer is paused, and plays
    /// what was prepared for that clock.
    fn advance(&mut self, controller: &mut MidiController<impl MidiOut>) {
        if !self.on_pause {
            self.step += 1;
            controller.update();
        }
    }

    /// Returns `true` if the sequencer is currently running, `false` otherwise.
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Returns `true` if the sequencer is currently paused, `false` otherwise.
    pub fn is_paused(&self) -> bool {
        self.on_pause
    }

    /// Internal MIDI input handler.
    ///
    /// This function is not intended to be called directly by users.
    /// Instead, users should implement [`Conductor::handle_input`] for their custom input handler logic.
    ///
    /// `handle_input` is used internally to enable code reuse across platforms and unify MIDI input processing.
    pub fn handle_input(
        &mut self,
        input_id: usize,
        conductor: &mut impl Conductor,
        controller: &mut MidiController<impl MidiOut>,
        input_queue: InputQueue,
    ) {
        let paused = self.is_paused();
        input_queue
            .into_iter()
            .flat_map(|input| conductor.handle_input(input_id, input, self))
            .for_each(|instruction| {
                if !paused || matches!(instruction, Instruction::MidiMessage { .. }) {
                    controller.execute(instruction);
                }
            });
    }
}
