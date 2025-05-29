mod common;

use std::{cell::RefCell, rc::Rc, time::Instant};

use common::*;
use mseq_core::*;
use std::collections::HashMap;

struct DebugInputConductor {
    midi_out: Rc<RefCell<DebugMidiOutInner>>,
}

impl Conductor for DebugInputConductor {
    fn init(&mut self, _context: &mut Context) {}

    fn update(&mut self, context: &mut Context) -> Vec<Instruction> {
        if context.get_step() == 30 {
            context.quit();
            return vec![];
        }

        // Check forwarding worked
        if (21..=24).contains(&context.get_step()) {
            assert!(
                self.midi_out.borrow().notes_on.contains_key(&(
                    1,
                    MidiNote {
                        note: Note::CS,
                        octave: 4,
                        vel: 160,
                    }
                    .midi_value()
                ))
            );
        } else {
            assert!(self.midi_out.borrow().notes_on.is_empty());
        }

        vec![]
    }

    fn handle_input(
        &mut self,
        channel_id: u8,
        input: mseq_core::MidiMessage,
        _context: &Context,
    ) -> Vec<Instruction> {
        match input {
            mseq_core::MidiMessage::NoteOff { note } => {
                vec![Instruction::MidiMessage {
                    channel_id,
                    midi_message: MidiMessage::NoteOff {
                        note: note.transpose(3),
                    },
                }]
            }
            mseq_core::MidiMessage::NoteOn { note } => {
                vec![Instruction::MidiMessage {
                    channel_id,
                    midi_message: MidiMessage::NoteOn {
                        note: note.transpose(3),
                    },
                }]
            }
            _ => vec![],
        }
    }
}

fn input_test_simulation(ctx: &Context, input_queue: &mut InputQueue) {
    if ctx.get_step() == 20 {
        input_queue.push_back((
            1,
            MidiMessage::NoteOn {
                note: MidiNote {
                    note: Note::AS,
                    octave: 3,
                    vel: 160,
                },
            },
        ));
    } else if ctx.get_step() == 24 {
        input_queue.push_back((
            1,
            MidiMessage::NoteOff {
                note: MidiNote {
                    note: Note::AS,
                    octave: 3,
                    vel: 160,
                },
            },
        ));
    }
}

fn test_conductor_with_input<T: MidiOut>(
    mut conductor: impl Conductor,
    mut midi_controller: MidiController<T>,
    input_simulation: impl Fn(&Context, &mut InputQueue),
) {
    let mut ctx = Context::new();
    let mut input_queue = InputQueue::new();
    conductor.init(&mut ctx);
    while ctx.is_running() {
        // Simulate incoming input
        input_simulation(&ctx, &mut input_queue);

        ctx.process_pre_tick(&mut conductor, &mut midi_controller);
        ctx.process_post_tick(&mut midi_controller);

        // Simulate input handling
        ctx.handle_inputs(&mut conductor, &mut midi_controller, &mut input_queue);
    }
    midi_controller.stop_all_notes(None);
    midi_controller.stop();
}

#[test]
fn test_input() {
    let debug_conn = Rc::new(RefCell::new(DebugMidiOutInner {
        notes_on: HashMap::new(),
        start_timestamp: Instant::now(),
    }));
    let midi = MidiController::new(DebugMidiOut(debug_conn.clone()));
    let conductor = DebugInputConductor {
        midi_out: debug_conn,
    };
    test_conductor_with_input(conductor, midi, input_test_simulation);
}
