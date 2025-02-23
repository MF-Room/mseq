use std::assert;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Instant;

use super::common::DebugMidiOut;
use super::common::DebugMidiOutInner;
use crate::Conductor;
use crate::Context;
use crate::MidiController;
use crate::MidiNote;
use crate::MidiOut;
use crate::Note;
use crate::Track;

#[test]
fn play_note() {
    let debug_conn = Rc::new(RefCell::new(DebugMidiOutInner {
        notes_on: HashMap::new(),
        start_timestamp: Instant::now(),
    }));

    let mut controller = MidiController::new(DebugMidiOut(debug_conn.clone()));
    controller.start();

    let note = MidiNote::new(crate::Note::B, 3, 21);
    controller.play_note(note, 3, 5);
    controller.send_clock();
    assert!(debug_conn.borrow().notes_on.is_empty());
    controller.update(1);
    assert!(debug_conn.borrow().notes_on.len() == 1);

    controller.send_clock();
    controller.update(2);
    assert!(debug_conn.borrow().notes_on.len() == 1);

    controller.send_clock();
    controller.update(3);
    assert!(debug_conn.borrow().notes_on.len() == 1);

    controller.send_clock();
    controller.update(4);
    assert!(debug_conn.borrow().notes_on.is_empty());

    controller.stop();
}

struct DebugConductor1(Rc<RefCell<DebugMidiOutInner>>);

impl Conductor for DebugConductor1 {
    fn init(&mut self, _context: &mut Context<impl MidiOut>) {}

    fn update(&mut self, context: &mut Context<impl MidiOut>) {
        if context.step == 0 {
            let note = MidiNote::new(crate::Note::B, 3, 21);
            context.midi.play_note(note, 5, 1);
        } else if context.step == 10 {
            context.quit();
        }
        if (1..=5).contains(&context.step) {
            assert!(self.0.borrow().notes_on.len() == 1);
        } else {
            assert!(self.0.borrow().notes_on.is_empty());
        }
    }
}

#[test]
fn play_note_conductor() {
    let debug_conn = Rc::new(RefCell::new(DebugMidiOutInner {
        notes_on: HashMap::new(),
        start_timestamp: Instant::now(),
    }));
    let midi = MidiController::new(DebugMidiOut(debug_conn.clone()));
    let conductor = DebugConductor1(debug_conn);
    super::common::test_conductor(conductor, midi);
}

struct DebugConductor2(Rc<RefCell<DebugMidiOutInner>>);

impl Conductor for DebugConductor2 {
    fn init(&mut self, _context: &mut Context<impl MidiOut>) {}

    fn update(&mut self, context: &mut Context<impl MidiOut>) {
        if context.step == 0 {
            let note = MidiNote::new(crate::Note::B, 10, 21);
            context.midi.play_note(note, 10, 1);
            context.midi.start_note(note, 3);
        } else if context.step == 5 {
            context.quit();
        }

        if (1..=5).contains(&context.step) {
            assert!(self.0.borrow().notes_on.len() == 2);
        }
    }
}

#[test]
fn notes_stop_on_quit() {
    let debug_conn = Rc::new(RefCell::new(DebugMidiOutInner {
        notes_on: HashMap::new(),
        start_timestamp: Instant::now(),
    }));
    let midi = MidiController::new(DebugMidiOut(debug_conn.clone()));
    let conductor = DebugConductor2(debug_conn.clone());
    super::common::test_conductor(conductor, midi);
    assert!(debug_conn.borrow().notes_on.is_empty());
}

struct DebugConductor3 {
    conn: Rc<RefCell<DebugMidiOutInner>>,
    track: crate::DeteTrack,
}

impl Conductor for DebugConductor3 {
    fn init(&mut self, _context: &mut Context<impl MidiOut>) {}

    fn update(&mut self, context: &mut Context<impl MidiOut>) {
        if context.step == 74 {
            context.quit();
        } else {
            if context.step == 0 {
                self.track.transpose(Some(Note::C));
            }

            if context.step == 48 {
                self.track.transpose(Some(Note::G));
            }

            if context.step == 1 {
                assert!(self
                    .conn
                    .borrow()
                    .notes_on
                    .contains_key(&(1, MidiNote::midi_value(&MidiNote::new(Note::C, 5, 89)))));
            }

            if context.step == 25 {
                assert!(self
                    .conn
                    .borrow()
                    .notes_on
                    .contains_key(&(1, MidiNote::midi_value(&MidiNote::new(Note::DS, 5, 89)))));
            }

            if context.step == 49 {
                assert!(self
                    .conn
                    .borrow()
                    .notes_on
                    .contains_key(&(1, MidiNote::midi_value(&MidiNote::new(Note::G, 4, 89)))));
            }

            if context.step == 73 {
                assert!(self
                    .conn
                    .borrow()
                    .notes_on
                    .contains_key(&(1, MidiNote::midi_value(&MidiNote::new(Note::AS, 4, 89)))));
            }

            context.midi.play_track(&mut self.track);
        }
    }
}

#[test]
fn dete_track_transpose() {
    let debug_conn = Rc::new(RefCell::new(DebugMidiOutInner {
        notes_on: HashMap::new(),
        start_timestamp: Instant::now(),
    }));
    let midi = MidiController::new(DebugMidiOut(debug_conn.clone()));
    let conductor = DebugConductor3 {
        conn: debug_conn,
        track: crate::DeteTrack::new(
            48,
            vec![
                (MidiNote::new(Note::A, 4, 89), 0, 12),
                (MidiNote::new(Note::C, 5, 89), 24, 12),
            ],
            Note::A,
            1,
            "test_transpose",
        ),
    };
    super::common::test_conductor(conductor, midi);
}
