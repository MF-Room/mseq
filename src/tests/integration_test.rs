use std::assert;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Instant;

use super::common::DebugMidiConnection;
use super::common::DebugMidiConnectionInner;
use crate::Conductor;
use crate::Context;
use crate::MidiConnection;
use crate::MidiController;
use crate::MidiNote;

#[test]
fn test_play_note() {
    let debug_conn = Rc::new(RefCell::new(DebugMidiConnectionInner {
        notes_on: HashMap::new(),
        start_timestamp: Instant::now(),
    }));

    let mut controller = MidiController::new(DebugMidiConnection(debug_conn.clone()));
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

struct DebugConductor1(Rc<RefCell<DebugMidiConnectionInner>>);

impl Conductor for DebugConductor1 {
    fn init(&mut self, _context: &mut Context<impl MidiConnection>) {}

    fn update(&mut self, context: &mut Context<impl MidiConnection>) {
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
fn test_play_note_conductor() {
    let debug_conn = Rc::new(RefCell::new(DebugMidiConnectionInner {
        notes_on: HashMap::new(),
        start_timestamp: Instant::now(),
    }));
    let midi = MidiController::new(DebugMidiConnection(debug_conn.clone()));
    let conductor = DebugConductor1(debug_conn);
    super::common::test_conductor(conductor, midi);
}

struct DebugConductor2(Rc<RefCell<DebugMidiConnectionInner>>);

impl Conductor for DebugConductor2 {
    fn init(&mut self, _context: &mut Context<impl MidiConnection>) {}

    fn update(&mut self, context: &mut Context<impl MidiConnection>) {
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
fn test_notes_stop_on_quit() {
    let debug_conn = Rc::new(RefCell::new(DebugMidiConnectionInner {
        notes_on: HashMap::new(),
        start_timestamp: Instant::now(),
    }));
    let midi = MidiController::new(DebugMidiConnection(debug_conn.clone()));
    let conductor = DebugConductor2(debug_conn.clone());
    super::common::test_conductor(conductor, midi);
    assert!(debug_conn.borrow().notes_on.is_empty());
}
