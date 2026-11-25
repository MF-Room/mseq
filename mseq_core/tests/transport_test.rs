//! What goes on the wire around transport changes, in master and in slave mode.

use mseq_core::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// A MIDI output that records every message it sends, in order.
#[derive(Clone, Default)]
struct WireOut(Rc<RefCell<Vec<MidiMessage>>>);

impl WireOut {
    /// The messages sent since the last call.
    fn take(&self) -> Vec<MidiMessage> {
        std::mem::take(&mut self.0.borrow_mut())
    }

    fn push(&self, message: MidiMessage) -> Result<(), String> {
        self.0.borrow_mut().push(message);
        Ok(())
    }
}

impl MidiOut for WireOut {
    type Error = String;
    fn send_start(&mut self) -> Result<(), String> {
        self.push(MidiMessage::Start)
    }
    fn send_continue(&mut self) -> Result<(), String> {
        self.push(MidiMessage::Continue)
    }
    fn send_stop(&mut self) -> Result<(), String> {
        self.push(MidiMessage::Stop)
    }
    fn send_clock(&mut self) -> Result<(), String> {
        self.push(MidiMessage::Clock)
    }
    fn send_note_on(&mut self, channel: u8, note: u8, velocity: u8) -> Result<(), String> {
        self.push(MidiMessage::NoteOn {
            channel,
            note: MidiNote::from_midi_value(note, velocity),
        })
    }
    fn send_note_off(&mut self, channel: u8, note: u8) -> Result<(), String> {
        self.push(MidiMessage::NoteOff {
            channel,
            note: MidiNote::from_midi_value(note, 0),
        })
    }
    fn send_cc(&mut self, channel: u8, controller: u8, value: u8) -> Result<(), String> {
        self.push(MidiMessage::CC {
            channel,
            controller,
            value,
        })
    }
    fn send_pc(&mut self, channel: u8, value: u8) -> Result<(), String> {
        self.push(MidiMessage::PC { channel, value })
    }
    fn send_pitch_bend(&mut self, channel: u8, value: u16) -> Result<(), String> {
        self.push(MidiMessage::PitchBend { channel, value })
    }
}

const CHANNEL: u8 = 1;
/// The steps the conductors play a note on, and how long the note lasts. Notes overlap, so that
/// one is always sounding.
const EVERY: u32 = 4;
const LEN: u32 = 6;

/// The note played at `step`, a different one for each step played.
fn note(step: u32) -> MidiNote {
    MidiNote::from_midi_value(48 + (step / EVERY % 32) as u8, 100)
}

fn on(step: u32) -> MidiMessage {
    MidiMessage::NoteOn {
        channel: CHANNEL,
        note: note(step),
    }
}

fn off(step: u32) -> MidiMessage {
    MidiMessage::NoteOff {
        channel: CHANNEL,
        note: MidiNote::from_midi_value(note(step).midi_value(), 0),
    }
}

fn play(step: u32) -> Vec<Instruction> {
    if step.is_multiple_of(EVERY) {
        vec![Instruction::PlayNote {
            midi_note: note(step),
            len: LEN,
            channel_id: CHANNEL,
        }]
    } else {
        vec![]
    }
}

/// Checks that every note-on of `wire` has exactly one note-off after it.
fn assert_balanced(wire: &[MidiMessage]) {
    let mut sounding = HashMap::<(u8, u8), u32>::new();
    for message in wire {
        match *message {
            MidiMessage::NoteOn { channel, note } => {
                *sounding.entry((channel, note.midi_value())).or_default() += 1
            }
            MidiMessage::NoteOff { channel, note } => {
                let count = sounding.entry((channel, note.midi_value())).or_default();
                assert!(
                    *count > 0,
                    "note-off with no note-on: {message:?} in {wire:?}"
                );
                *count -= 1;
            }
            _ => {}
        }
    }
    sounding.retain(|_, count| *count > 0);
    assert!(
        sounding.is_empty(),
        "notes left on: {sounding:?} in {wire:?}"
    );
}

/// Plays `step(context.get_step())`, and starts in `init` as most conductors do.
struct Player {
    context_changes: u32,
}

impl Conductor for Player {
    fn init(&mut self, context: &mut Context) -> Vec<Instruction> {
        context.set_bpm(90);
        context.start();
        vec![]
    }

    fn update(&mut self, context: &mut Context) -> Vec<Instruction> {
        play(context.get_step())
    }

    fn context_changed(&mut self, _context: &Context) {
        self.context_changes += 1;
    }
}

/// A slave driven through Start, Stop, Continue and a Start while playing, as a clock source
/// would.
#[test]
fn slave_plays_on_the_clock_source_transport() {
    let wire = WireOut::default();
    let mut controller = MidiController::new(wire.clone());
    let mut conductor = Player { context_changes: 0 };
    let mut ctx = Context::default();
    use MidiMessage::{Clock, Continue, Start, Stop};

    // The clock source owns the transport: the Start init asked for is not sent, and the BPM it
    // set is not the clock source's.
    ctx.init_slave(&mut conductor, &mut controller);
    assert_eq!(wire.take(), []);
    assert!(ctx.is_paused());
    assert_eq!(ctx.get_bpm(), None);
    assert_eq!(conductor.context_changes, 1);

    let mut all = vec![];
    let mut send = |ctx: &mut Context, conductor: &mut Player, message| {
        ctx.process_transport(conductor, &mut controller, message);
        let sent = wire.take();
        all.extend(sent.iter().copied());
        sent
    };

    // Clocks before the Start are sent on, and play nothing.
    assert_eq!(send(&mut ctx, &mut conductor, Clock), [Clock]);
    assert_eq!(ctx.get_step(), 0);

    // The first Clock after Start plays step 0.
    assert_eq!(send(&mut ctx, &mut conductor, Start), [Start]);
    assert_eq!(conductor.context_changes, 2);
    assert_eq!(send(&mut ctx, &mut conductor, Clock), [Clock, on(0)]);
    for step in 1..8 {
        let mut expected = vec![Clock];
        if step >= LEN && (step - LEN).is_multiple_of(EVERY) {
            expected.push(off(step - LEN));
        }
        expected.extend(play(step).iter().map(|_| on(step)));
        assert_eq!(
            send(&mut ctx, &mut conductor, Clock),
            expected,
            "step {step}"
        );
    }
    assert_eq!(ctx.get_step(), 8);

    // Stop turns off the note sounding (step 4's), and only it.
    assert_eq!(send(&mut ctx, &mut conductor, Stop), [off(4), Stop]);
    assert_eq!(conductor.context_changes, 3);
    for _ in 0..3 {
        assert_eq!(send(&mut ctx, &mut conductor, Clock), [Clock]);
    }
    assert_eq!(ctx.get_step(), 8);

    // The first Clock after Continue plays the step where Stop left off, and its note ends.
    assert_eq!(send(&mut ctx, &mut conductor, Continue), [Continue]);
    assert_eq!(send(&mut ctx, &mut conductor, Clock), [Clock, on(8)]);
    for _ in 9..14 {
        send(&mut ctx, &mut conductor, Clock);
    }
    assert_eq!(send(&mut ctx, &mut conductor, Clock), [Clock, off(8)]);
    assert_eq!(ctx.get_step(), 15);

    // A Start while playing: the next Clock plays step 0, and the note sounding (step 12's, on
    // until step 18) still ends after its length.
    assert_eq!(send(&mut ctx, &mut conductor, Start), [Start]);
    assert_eq!(send(&mut ctx, &mut conductor, Clock), [Clock, on(0)]);
    assert_eq!(send(&mut ctx, &mut conductor, Clock), [Clock]);
    assert_eq!(send(&mut ctx, &mut conductor, Clock), [Clock]);
    assert_eq!(send(&mut ctx, &mut conductor, Clock), [Clock, off(12)]);
    assert_eq!(ctx.get_step(), 4);

    // Quitting turns off what still sounds, once.
    controller.finish();
    all.extend(wire.take());
    assert_eq!(all.last(), Some(&Stop));
    assert_balanced(&all);
}

/// Several Clocks in a row each play a step.
#[test]
fn slave_plays_a_step_per_clock() {
    let wire = WireOut::default();
    let mut controller = MidiController::new(wire.clone());
    let mut conductor = Player { context_changes: 0 };
    let mut ctx = Context::default();
    ctx.init_slave(&mut conductor, &mut controller);
    for message in [MidiMessage::Start, MidiMessage::Clock, MidiMessage::Clock] {
        ctx.process_transport(&mut conductor, &mut controller, message);
    }
    assert_eq!(ctx.get_step(), 2);
}

/// Toggles the transport from `update` on the given updates, as a start/stop button does, and
/// plays only while running.
struct Toggler {
    updates: u32,
    toggles: Vec<u32>,
}

impl Conductor for Toggler {
    fn init(&mut self, _context: &mut Context) -> Vec<Instruction> {
        vec![]
    }

    fn update(&mut self, context: &mut Context) -> Vec<Instruction> {
        self.updates += 1;
        if self.toggles.contains(&self.updates) {
            if context.is_paused() {
                context.start();
            } else {
                context.pause();
            }
        }
        if context.is_paused() {
            return vec![];
        }
        play(context.get_step())
    }
}

/// In master mode, transport from `update` applies to the tick it prepares.
#[test]
fn master_applies_the_transport_of_update_to_its_tick() {
    let wire = WireOut::default();
    let mut controller = MidiController::new(wire.clone());
    let mut conductor = Toggler {
        updates: 0,
        toggles: vec![3, 10],
    };
    let mut ctx = Context::default();
    ctx.init(&mut conductor, &mut controller);
    let mut tick = |ctx: &mut Context, conductor: &mut Toggler| {
        ctx.process_pre_tick(conductor, &mut controller);
        ctx.process_post_tick(&mut controller);
        wire.take()
    };
    use MidiMessage::{Clock, Start, Stop};

    assert_eq!(tick(&mut ctx, &mut conductor), [Clock]);
    assert_eq!(tick(&mut ctx, &mut conductor), [Clock]);

    // Started on the third update: Start goes out before the Clock, which plays step 0.
    assert_eq!(tick(&mut ctx, &mut conductor), [Start, Clock, on(0)]);
    for _ in 1..7 {
        tick(&mut ctx, &mut conductor);
    }
    assert_eq!(ctx.get_step(), 7);

    // Paused on the tenth update: the off of the note sounding (step 4's) and Stop go out before
    // the Clock, and the step stays.
    assert_eq!(tick(&mut ctx, &mut conductor), [off(4), Stop, Clock]);
    assert_eq!(ctx.get_step(), 7);
}

/// Plays the given instructions on the first update.
struct Once(Vec<Instruction>);

impl Conductor for Once {
    fn init(&mut self, context: &mut Context) -> Vec<Instruction> {
        context.start();
        vec![]
    }

    fn update(&mut self, _context: &mut Context) -> Vec<Instruction> {
        std::mem::take(&mut self.0)
    }

    fn handle_input(
        &mut self,
        _input_id: usize,
        _input: MidiMessage,
        _context: &Context,
    ) -> Vec<Instruction> {
        std::mem::take(&mut self.0)
    }
}

/// A note started and stopped before the clock that would send it is never sent.
#[test]
fn a_note_stopped_before_it_starts_is_not_sent() {
    let wire = WireOut::default();
    let mut controller = MidiController::new(wire.clone());
    let g = MidiNote::new(Note::G, 4, 100);
    let mut conductor = Once(vec![
        Instruction::StartNote {
            midi_note: g,
            channel_id: CHANNEL,
        },
        Instruction::StopNote {
            midi_note: g,
            channel_id: CHANNEL,
        },
    ]);
    let mut ctx = Context::default();
    ctx.init(&mut conductor, &mut controller);
    ctx.process_pre_tick(&mut conductor, &mut controller);
    ctx.process_post_tick(&mut controller);
    controller.finish();
    assert_eq!(
        wire.take(),
        [MidiMessage::Start, MidiMessage::Clock, MidiMessage::Stop]
    );
}

/// A raw message on a channel outside 1 to 16 is dropped, as every other instruction is.
#[test]
fn raw_messages_need_a_valid_channel() {
    let wire = WireOut::default();
    let mut controller = MidiController::new(wire.clone());
    let cc = |channel| MidiMessage::CC {
        channel,
        controller: 7,
        value: 100,
    };
    let mut conductor = Once(
        [0, 1, 16, 17]
            .map(|channel| Instruction::MidiMessage {
                midi_message: cc(channel),
            })
            .to_vec(),
    );
    let mut ctx = Context::default();
    ctx.handle_input(
        0,
        &mut conductor,
        &mut controller,
        InputQueue::from([cc(1)]),
    );
    assert_eq!(wire.take(), [cc(1), cc(16)]);
}
