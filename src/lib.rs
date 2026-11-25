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
//!   Start/Stop/Continue messages, dynamically adjusting BPM to match the clock source. Each
//!   clock pulse plays one step: the first one after a Start plays step 0, and the first one
//!   after a Continue plays the step the Stop left off at. The BPM is unknown (`None`) until the
//!   first beat of clock is measured, and again after 500 ms without a clock pulse.
//!
//! ## Usage
//!
//! The entry point of the crate is the [`run`] function:
//!
//! ```no_run
//! use mseq::{run, Conductor, Context, Instruction, MidiNote, Note};
//!
//! struct MyConductor;
//!
//! impl Conductor for MyConductor {
//!     fn init(&mut self, ctx: &mut Context) -> Vec<Instruction> {
//!         ctx.set_bpm(120);
//!         // The sequencer starts paused: nothing plays until you call start().
//!         ctx.start();
//!         vec![]
//!     }
//!
//!     fn update(&mut self, ctx: &mut Context) -> Vec<Instruction> {
//!         // update() runs on every MIDI clock pulse, so there are 24 steps per quarter note.
//!         if ctx.get_step() % 24 == 0 {
//!             return vec![Instruction::PlayNote {
//!                 midi_note: MidiNote::new(Note::C, 4, 100),
//!                 len: 12,
//!                 channel_id: 1,
//!             }];
//!         }
//!         vec![]
//!     }
//! }
//!
//! fn main() -> Result<(), mseq::MSeqError> {
//!     let conductor = MyConductor;
//!     let out_port = None;        // Ask user for output port
//!     let midi_in = Vec::new();   // Run standalone (no input, master clock/transport)
//!     run(conductor, out_port, midi_in)
//! }
//! ```
//!
//! ## Architecture
//!
//! See the [schematic](https://github.com/MF-Room/mseq/blob/main/README.md#architecture)
//! of what you implement and what the engine does with it.
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
use std::sync::{Arc, Mutex};
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
/// - `midi_in`: List of [`MidiInParam`], one per MIDI input to open. Each input gets its own queue,
///   and the input's 0-based position in this list is passed to [`Conductor::handle_input`] as the
///   `input_id`. An empty list runs the sequencer standalone (no input).
///   At most one input acts as the clock/transport source: the first one with `slave` set to `true`.
///
/// # Behavior
/// - **No input (`midi_in` empty)** → sequencer runs with its internal clock and transport,
///   generating MIDI clock and transport messages, but ignoring any external MIDI input.
/// - **Master mode** (no input marked `slave`) → sequencer generates its own MIDI clock and transport
///   messages while also handling incoming MIDI events (except for clock/transport).
/// - **Slave mode** (the first `slave` input) → sequencer synchronizes to that input's MIDI clock,
///   Start/Stop/Continue messages, and dynamically adjusts BPM to match the external clock source.
///   Each clock pulse plays one step, and the transport the conductor asks for in
///   [`Conductor::init`] is not sent: the clock source owns it.
///   [`Context::get_bpm`] returns `None` until the first beat of clock is measured, and again
///   after 500 ms without a clock pulse. Changes to the context between two clock pulses are
///   reported through [`Conductor::context_changed`]. The remaining inputs are handled as
///   message-only inputs.
///
/// # Errors
/// Returns an [`MSeqError`] if a MIDI port cannot be opened, if MIDI input/output fails, or if track
/// parsing encounters an error.
pub fn run(
    conductor: impl Conductor + std::marker::Send + 'static,
    out_port: Option<u32>,
    midi_in: Vec<MidiInParam>,
) -> Result<(), MSeqError> {
    let midi_out = StdMidiOut::new(out_port)?;
    let midi_controller = MidiController::new(midi_out);
    let ctx = Context::default();

    if midi_in.is_empty() {
        return run_no_input(ctx, midi_controller, conductor);
    }

    // At most one input is the clock/transport source: the first one marked as slave.
    let slave_idx = midi_in.iter().position(|p| p.slave);
    let slave_count = midi_in.iter().filter(|p| p.slave).count();
    if slave_count > 1 {
        log::warn!(
            "{slave_count} MIDI inputs are marked as slave, but a single clock/transport source is \
             supported: using input {} as the clock source, the others are treated as message-only inputs.",
            slave_idx.unwrap()
        );
    }

    let run = Arc::new(Mutex::new((conductor, midi_controller, ctx)));

    // Connect every input, keeping the connections alive for the whole run.
    let connections = midi_in
        .into_iter()
        .enumerate()
        .map(|(input_id, params)| connect(input_id, params, Some(input_id) == slave_idx))
        .collect::<Result<Vec<_>, _>>()?;

    // Spawn one consumer thread per input, each draining its own queue.
    for (input_id, conn) in connections.iter().enumerate() {
        let run_consumer = run.clone();
        let channel = conn.message.clone();
        thread::spawn(move || {
            loop {
                // Wait for messages, then swap them out so the callback isn't blocked
                // while we process.
                let pending = {
                    let mut queue = channel.queue.lock().unwrap();
                    while queue.is_empty() {
                        queue = channel.condvar.wait(queue).unwrap();
                    }
                    core::mem::take(&mut *queue)
                };
                let mut r = run_consumer.lock().unwrap();
                let (ref mut conductor, ref mut controller, ref mut ctx) = *r;
                ctx.handle_input(input_id, conductor, controller, pending);
            }
        });
    }

    let slave_system = connections.iter().find_map(|c| c.slave_system.clone());
    if let Some(slave) = slave_system {
        run_slave(run, slave)
    } else {
        run_master(run)
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
    slave: NotifyQueue,
) -> Result<(), MSeqError> {
    {
        let mut r = run.lock().unwrap();
        let (ref mut conductor, ref mut controller, ref mut ctx) = *r;
        // Paused until the clock source's Start, with the BPM unknown until a beat is measured.
        ctx.init_slave(conductor, controller);
    }

    let mut meter = BpmMeter::default();
    // Time of the last clock, while the external clock is running.
    let mut last_clock: Option<Instant> = None;
    'run: loop {
        let Some(messages) = take_sys_messages(&slave, last_clock.map(|t| t + CLOCK_TIMEOUT))
        else {
            // The external clock stopped: the BPM is unknown until it runs for a beat again.
            last_clock = None;
            meter.reset();
            let mut r = run.lock().unwrap();
            let (ref mut conductor, _, ref mut ctx) = *r;
            ctx.clear_bpm();
            // No update runs until the clock is back, so tell the conductor now.
            conductor.context_changed(ctx);
            continue;
        };
        let received = Instant::now();

        // Apply the messages in order with sys_queue released, so the callback isn't blocked.
        // Each clock plays a step, even when several queued up.
        let mut r = run.lock().unwrap();
        let (ref mut conductor, ref mut controller, ref mut ctx) = *r;
        for message in messages {
            if message == MidiMessage::Clock {
                last_clock = Some(received);
                // Before the step, so that its update sees it.
                if let Some(bpm) = meter.clock(received) {
                    ctx.set_bpm(bpm);
                }
            }
            ctx.process_transport(conductor, controller, message);
            if !ctx.is_running() {
                break 'run;
            }
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

/// How long the external clock can stay silent before the slave loop considers it stopped and
/// the BPM unknown. 500 ms between two clocks is a tempo of 5 BPM, far slower than any music and
/// far longer than any clock jitter.
const CLOCK_TIMEOUT: Duration = Duration::from_millis(500);

/// Number of MIDI clocks in a beat (quarter note): the interval the BPM is measured over.
const CLOCKS_PER_BEAT: u32 = 24;

/// Measures the BPM of an external MIDI clock, from the time one beat takes.
#[derive(Default)]
struct BpmMeter {
    /// Time of the clock that opened the current beat, if one did.
    start: Option<Instant>,
    /// Number of clocks received since `start`.
    clocks: u32,
}

impl BpmMeter {
    /// Records a clock received at `now`. Returns the BPM on the clock that ends a beat, if it
    /// is in range (1 to 255): an out-of-range reading leaves the BPM as it was.
    fn clock(&mut self, now: Instant) -> Option<u8> {
        let Some(start) = self.start else {
            // The first clock only opens the beat.
            self.start = Some(now);
            self.clocks = 0;
            return None;
        };
        self.clocks += 1;
        if self.clocks < CLOCKS_PER_BEAT {
            return None;
        }
        self.start = Some(now);
        self.clocks = 0;
        // bpm = 60000 ms / beat
        let duration = now.duration_since(start).as_millis();
        60000_u128
            .checked_div(duration)
            .and_then(|bpm| u8::try_from(bpm).ok())
            .filter(|&bpm| bpm >= 1)
    }

    /// Forgets the current beat: the next clock opens a new one.
    fn reset(&mut self) {
        *self = Self::default();
    }
}

/// Waits until the slave queue holds messages and takes them all. With a `deadline`, returns
/// `None` if it passes with nothing queued.
fn take_sys_messages(slave: &NotifyQueue, deadline: Option<Instant>) -> Option<InputQueue> {
    let mut queue = slave.queue.lock().unwrap();
    while queue.is_empty() {
        match deadline {
            None => queue = slave.condvar.wait(queue).unwrap(),
            Some(deadline) => {
                let remaining = deadline.saturating_duration_since(Instant::now());
                if remaining.is_zero() {
                    return None;
                }
                queue = slave.condvar.wait_timeout(queue, remaining).unwrap().0;
            }
        }
    }
    Some(core::mem::take(&mut *queue))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Sends one beat (24 clocks after the first) at `bpm` to `meter`, starting at `start`.
    /// Returns what each clock returned and the time of the last clock.
    fn beat(meter: &mut BpmMeter, start: Instant, bpm: u64) -> (Vec<Option<u8>>, Instant) {
        let period = Duration::from_micros(60_000_000 / 24 / bpm);
        let readings = (1..=CLOCKS_PER_BEAT)
            .map(|i| meter.clock(start + period * i))
            .collect();
        (readings, start + period * CLOCKS_PER_BEAT)
    }

    #[test]
    fn bpm_meter_measures_a_beat() {
        let mut meter = BpmMeter::default();
        let t0 = Instant::now();
        // The first clock only opens the beat.
        assert_eq!(meter.clock(t0), None);
        let (readings, t1) = beat(&mut meter, t0, 120);
        assert!(readings[..23].iter().all(Option::is_none));
        assert_eq!(readings[23], Some(120));

        // The next beat starts where this one ended.
        let (readings, _) = beat(&mut meter, t1, 90);
        assert_eq!(readings[23], Some(90));
    }

    #[test]
    fn bpm_meter_ignores_the_wait_before_the_first_clock() {
        let mut meter = BpmMeter::default();
        // A meter created at t0 whose first clock only comes 3 s later.
        let first = Instant::now() + Duration::from_secs(3);
        assert_eq!(meter.clock(first), None);
        let (readings, _) = beat(&mut meter, first, 120);
        assert_eq!(readings[23], Some(120));
    }

    #[test]
    fn bpm_meter_reset_restarts_the_beat() {
        let mut meter = BpmMeter::default();
        let t0 = Instant::now();
        let period = Duration::from_micros(60_000_000 / 24 / 120);
        for i in 0..10 {
            assert_eq!(meter.clock(t0 + period * i), None);
        }
        meter.reset();

        // The clock resumes 2 s later: the gap is not part of the next beat.
        let resume = t0 + Duration::from_secs(2);
        assert_eq!(meter.clock(resume), None);
        let (readings, _) = beat(&mut meter, resume, 120);
        assert!(readings[..23].iter().all(Option::is_none));
        assert_eq!(readings[23], Some(120));
    }

    #[test]
    fn bpm_meter_skips_out_of_range_readings() {
        let mut meter = BpmMeter::default();
        let t0 = Instant::now();
        assert_eq!(meter.clock(t0), None);
        // A 200 ms beat is 300 BPM, which does not fit in a u8.
        let period = Duration::from_millis(200) / CLOCKS_PER_BEAT;
        let last = (1..=CLOCKS_PER_BEAT)
            .map(|i| meter.clock(t0 + period * i))
            .last();
        assert_eq!(last, Some(None));
    }

    #[test]
    fn take_sys_messages_times_out_on_an_empty_queue() {
        let slave = NotifyQueue::new();
        let start = Instant::now();
        let timeout = Duration::from_millis(50);
        assert!(take_sys_messages(&slave, Some(start + timeout)).is_none());
        assert!(start.elapsed() >= timeout);
    }

    #[test]
    fn take_sys_messages_returns_messages_pushed_before_the_deadline() {
        let slave = NotifyQueue::new();
        let producer = slave.clone();
        let handle = thread::spawn(move || {
            thread::sleep(Duration::from_millis(20));
            producer.push(MidiMessage::Clock);
        });
        let messages = take_sys_messages(&slave, Some(Instant::now() + Duration::from_secs(5)));
        handle.join().unwrap();
        assert_eq!(messages, Some(InputQueue::from([MidiMessage::Clock])));
    }

    #[test]
    fn take_sys_messages_without_deadline_returns_queued_messages() {
        let slave = NotifyQueue::new();
        slave.push(MidiMessage::Start);
        slave.push(MidiMessage::Clock);
        assert_eq!(
            take_sys_messages(&slave, None),
            Some(InputQueue::from([MidiMessage::Start, MidiMessage::Clock]))
        );
        assert!(slave.queue.lock().unwrap().is_empty());
    }

    /// A MIDI output that drops everything.
    struct NullMidiOut;

    impl MidiOut for NullMidiOut {
        type Error = std::convert::Infallible;
        fn send_start(&mut self) -> Result<(), Self::Error> {
            Ok(())
        }
        fn send_continue(&mut self) -> Result<(), Self::Error> {
            Ok(())
        }
        fn send_stop(&mut self) -> Result<(), Self::Error> {
            Ok(())
        }
        fn send_clock(&mut self) -> Result<(), Self::Error> {
            Ok(())
        }
        fn send_note_on(&mut self, _: u8, _: u8, _: u8) -> Result<(), Self::Error> {
            Ok(())
        }
        fn send_note_off(&mut self, _: u8, _: u8) -> Result<(), Self::Error> {
            Ok(())
        }
        fn send_cc(&mut self, _: u8, _: u8, _: u8) -> Result<(), Self::Error> {
            Ok(())
        }
        fn send_pc(&mut self, _: u8, _: u8) -> Result<(), Self::Error> {
            Ok(())
        }
        fn send_pitch_bend(&mut self, _: u8, _: u16) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    /// Records `(is_paused, bpm)` on every `context_changed`.
    #[derive(Default)]
    struct RecordingConductor {
        changes: Vec<(bool, Option<u8>)>,
    }

    impl Conductor for RecordingConductor {
        fn init(&mut self, context: &mut Context) -> Vec<Instruction> {
            // Not the external clock's BPM: the slave loop clears it.
            context.set_bpm(90);
            vec![]
        }

        fn update(&mut self, _context: &mut Context) -> Vec<Instruction> {
            vec![]
        }

        fn context_changed(&mut self, context: &Context) {
            self.changes.push((context.is_paused(), context.get_bpm()));
        }
    }

    type SlaveState = (RecordingConductor, MidiController<NullMidiOut>, Context);

    /// Waits until `condition` holds on the state shared with `run_slave`, for at most 5 s.
    fn wait_until(run: &Arc<Mutex<SlaveState>>, condition: impl Fn(&SlaveState) -> bool) {
        let deadline = Instant::now() + Duration::from_secs(5);
        while !condition(&run.lock().unwrap()) {
            assert!(Instant::now() < deadline, "timed out waiting on run_slave");
            thread::sleep(Duration::from_millis(1));
        }
    }

    /// Waits for the conductor to have seen `n` context changes, and returns them.
    fn wait_for_changes(run: &Arc<Mutex<SlaveState>>, n: usize) -> Vec<(bool, Option<u8>)> {
        wait_until(run, |(conductor, _, _)| conductor.changes.len() >= n);
        run.lock().unwrap().0.changes.clone()
    }

    #[test]
    fn run_slave_tells_the_conductor_when_the_context_changes_between_steps() {
        let run = Arc::new(Mutex::new((
            RecordingConductor::default(),
            MidiController::new(NullMidiOut),
            Context::default(),
        )));
        let slave = NotifyQueue::new();
        let handle = {
            let (run, slave) = (run.clone(), slave.clone());
            thread::spawn(move || run_slave(run, slave))
        };

        // After init: paused, and the BPM init set is cleared.
        assert_eq!(wait_for_changes(&run, 1), [(true, None)]);

        slave.push(MidiMessage::Start);
        assert_eq!(wait_for_changes(&run, 2)[1], (false, None));

        // One beat of clock at 200 BPM. The BPM gets measured on a clock, where update sees
        // it, so the conductor is not told.
        let period = Duration::from_micros(60_000_000 / 24 / 200);
        let mut last_clock = Instant::now();
        for i in 0..=CLOCKS_PER_BEAT {
            if i > 0 {
                thread::sleep(period);
            }
            last_clock = Instant::now();
            slave.push(MidiMessage::Clock);
        }
        wait_until(&run, |(_, _, ctx)| ctx.get_bpm().is_some());
        assert_eq!(run.lock().unwrap().0.changes.len(), 2);

        // The clock stops: once it has been silent for the timeout, the BPM is unknown.
        assert_eq!(wait_for_changes(&run, 3)[2], (false, None));
        assert!(last_clock.elapsed() >= CLOCK_TIMEOUT);

        // A Stop with no clock after it, as from a master that stops its clock on stop.
        slave.push(MidiMessage::Stop);
        assert_eq!(wait_for_changes(&run, 4)[3], (true, None));

        // Quit on the next clock.
        run.lock().unwrap().2.quit();
        slave.push(MidiMessage::Clock);
        handle.join().unwrap().unwrap();
        assert_eq!(run.lock().unwrap().0.changes.len(), 4);
    }

    #[test]
    fn run_slave_plays_a_step_per_clock_when_clocks_queue_up() {
        let run = Arc::new(Mutex::new((
            RecordingConductor::default(),
            MidiController::new(NullMidiOut),
            Context::default(),
        )));
        let slave = NotifyQueue::new();
        let handle = {
            let (run, slave) = (run.clone(), slave.clone());
            thread::spawn(move || run_slave(run, slave))
        };
        wait_for_changes(&run, 1);

        // While the state is locked, run_slave cannot apply what it takes from the queue, so the
        // clocks queue up and it takes them together.
        {
            let _state = run.lock().unwrap();
            slave.push(MidiMessage::Start);
            for _ in 0..3 {
                slave.push(MidiMessage::Clock);
            }
        }
        wait_until(&run, |(_, _, ctx)| ctx.get_step() == 3);

        run.lock().unwrap().2.quit();
        slave.push(MidiMessage::Clock);
        handle.join().unwrap().unwrap();
        assert_eq!(run.lock().unwrap().2.get_step(), 4);
    }
}
