use std::assert;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Instant;

mod common;

use common::*;
use mseq_core::*;

struct DebugConductor1(Rc<RefCell<DebugMidiOutInner>>);

impl Conductor for DebugConductor1 {
    fn init(&mut self, context: &mut Context) -> Vec<Instruction> {
        context.start();
        vec![]
    }

    fn update(&mut self, context: &mut Context) -> Vec<Instruction> {
        let mut instructions = vec![];
        if context.get_step() == 0 {
            let note = MidiNote::new(mseq_core::Note::B, 3, 21);
            instructions.push(Instruction::PlayNote {
                midi_note: note,
                len: 5,
                channel_id: 1,
            });
        } else if context.get_step() == 10 {
            context.quit();
        }
        if (1..=5).contains(&context.get_step()) {
            assert!(self.0.borrow().notes_on.len() == 1);
        } else {
            assert!(self.0.borrow().notes_on.is_empty());
        }
        instructions
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
    test_conductor(conductor, midi);
}

struct DebugConductor2(Rc<RefCell<DebugMidiOutInner>>);

impl Conductor for DebugConductor2 {
    fn init(&mut self, context: &mut Context) -> Vec<Instruction> {
        context.start();
        vec![]
    }

    fn update(&mut self, context: &mut Context) -> Vec<Instruction> {
        let mut instructions = vec![];
        if context.get_step() == 0 {
            let note = MidiNote::new(crate::Note::B, 10, 21);
            instructions.push(Instruction::PlayNote {
                midi_note: note,
                len: 10,
                channel_id: 1,
            });
            instructions.push(Instruction::StartNote {
                midi_note: note,
                channel_id: 3,
            });
        } else if context.get_step() == 5 {
            context.quit();
        }

        if (1..=5).contains(&context.get_step()) {
            assert!(self.0.borrow().notes_on.len() == 2);
        }

        instructions
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
    test_conductor(conductor, midi);
    assert!(debug_conn.borrow().notes_on.is_empty());
}

struct DebugConductor3 {
    conn: Rc<RefCell<DebugMidiOutInner>>,
    track: crate::DeteTrack,
}

impl Conductor for DebugConductor3 {
    fn init(&mut self, context: &mut Context) -> Vec<Instruction> {
        context.start();
        vec![]
    }

    fn update(&mut self, context: &mut Context) -> Vec<Instruction> {
        if context.get_step() == 74 {
            context.quit();
            vec![]
        } else {
            if context.get_step() == 0 {
                self.track.transpose(Some(Note::C));
            }

            if context.get_step() == 48 {
                self.track.transpose(Some(Note::G));
            }

            if context.get_step() == 1 {
                assert!(
                    self.conn
                        .borrow()
                        .notes_on
                        .contains_key(&(1, MidiNote::midi_value(&MidiNote::new(Note::C, 5, 89))))
                );
            }

            if context.get_step() == 25 {
                assert!(
                    self.conn
                        .borrow()
                        .notes_on
                        .contains_key(&(1, MidiNote::midi_value(&MidiNote::new(Note::DS, 5, 89))))
                );
            }

            if context.get_step() == 49 {
                assert!(
                    self.conn
                        .borrow()
                        .notes_on
                        .contains_key(&(1, MidiNote::midi_value(&MidiNote::new(Note::G, 4, 89))))
                );
            }

            if context.get_step() == 73 {
                assert!(
                    self.conn
                        .borrow()
                        .notes_on
                        .contains_key(&(1, MidiNote::midi_value(&MidiNote::new(Note::AS, 4, 89))))
                );
            }

            self.track.play_step(context.get_step())
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
    test_conductor(conductor, midi);
}

pub fn test_conductor<T: MidiOut>(
    mut conductor: impl Conductor,
    mut midi_controller: MidiController<T>,
) {
    let mut ctx = Context::default();
    conductor.init(&mut ctx);
    while ctx.is_running() {
        ctx.process_pre_tick(&mut conductor, &mut midi_controller);
        // No sleep between each cycle because we're testing
        ctx.process_post_tick(&mut midi_controller);
    }
    midi_controller.stop_all_notes();
    midi_controller.stop();
}
