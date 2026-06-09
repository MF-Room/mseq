mod common;

use std::{cell::RefCell, rc::Rc, time::Instant};

use common::*;
use mseq_core::*;
use std::collections::HashMap;

/// Number of independent MIDI inputs simulated by the test.
const NUM_INPUTS: usize = 2;

struct DebugInputConductor {
    midi_out: Rc<RefCell<DebugMidiOutInner>>,
}

impl Conductor for DebugInputConductor {
    fn init(&mut self, context: &mut Context) -> Vec<Instruction> {
        context.start();
        vec![]
    }

    fn update(&mut self, context: &mut Context) -> Vec<Instruction> {
        if context.get_step() == 30 {
            context.quit();
            return vec![];
        }

        // Check forwarding worked: each input transposes by a different amount
        // (3 + input_id), so the same incoming note produces one distinct output
        // note per input. Here input 0 -> CS4 and input 1 -> D4.
        if (21..=24).contains(&context.get_step()) {
            let midi_out = self.midi_out.borrow();
            assert!(
                midi_out.notes_on.contains_key(&(
                    1,
                    MidiNote {
                        note: Note::CS,
                        octave: 4,
                        vel: 160,
                    }
                    .midi_value()
                ))
            );
            assert!(
                midi_out.notes_on.contains_key(&(
                    1,
                    MidiNote {
                        note: Note::D,
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
        input_id: usize,
        input: mseq_core::MidiMessage,
        _context: &Context,
    ) -> Vec<Instruction> {
        // Transpose by an input-dependent amount to prove that `input_id` is
        // correctly forwarded and that each input is handled independently.
        let semitones = 3 + input_id as i8;
        match input {
            mseq_core::MidiMessage::NoteOff { channel, note } => {
                vec![Instruction::MidiMessage {
                    midi_message: MidiMessage::NoteOff {
                        channel,
                        note: note.transpose(semitones),
                    },
                }]
            }
            mseq_core::MidiMessage::NoteOn { channel, note } => {
                vec![Instruction::MidiMessage {
                    midi_message: MidiMessage::NoteOn {
                        channel,
                        note: note.transpose(semitones),
                    },
                }]
            }
            _ => vec![],
        }
    }
}

fn input_test_simulation(ctx: &Context, _input_id: usize, input_queue: &mut InputQueue) {
    if ctx.get_step() == 20 {
        input_queue.push_back(MidiMessage::NoteOn {
            channel: 1,
            note: MidiNote {
                note: Note::AS,
                octave: 3,
                vel: 160,
            },
        });
    } else if ctx.get_step() == 24 {
        input_queue.push_back(MidiMessage::NoteOff {
            channel: 1,
            note: MidiNote {
                note: Note::AS,
                octave: 3,
                vel: 160,
            },
        });
    }
}

fn test_conductor_with_input<T: MidiOut>(
    mut conductor: impl Conductor,
    mut midi_controller: MidiController<T>,
    input_simulation: impl Fn(&Context, usize, &mut InputQueue),
) {
    let mut ctx = Context::default();
    // One independent queue per input.
    let mut input_queues: Vec<InputQueue> = (0..NUM_INPUTS).map(|_| InputQueue::new()).collect();
    conductor.init(&mut ctx);
    while ctx.is_running() {
        // Simulate incoming input on each input
        for (input_id, queue) in input_queues.iter_mut().enumerate() {
            input_simulation(&ctx, input_id, queue);
        }

        ctx.process_pre_tick(&mut conductor, &mut midi_controller);
        ctx.process_post_tick(&mut midi_controller);

        // Simulate input handling, one input (and its queue) at a time
        for (input_id, queue) in input_queues.iter_mut().enumerate() {
            ctx.handle_input(input_id, &mut conductor, &mut midi_controller, queue);
        }
    }
    midi_controller.finish();
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
