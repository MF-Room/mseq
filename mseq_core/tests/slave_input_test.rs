mod common;

use common::*;
use mseq_core::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::rc::Rc;
use std::time::Instant;

/// Number of independent MIDI inputs simulated by the test. Input 0 is the slave
/// (clock/transport source); the others are message-only inputs.
const NUM_INPUTS: usize = 2;
const SLAVE_INPUT: usize = 0;

/// Conductor that records every message delivered to `handle_input`, together with
/// the input it came from. Used to prove that transport messages from non-slave
/// inputs never reach the conductor.
struct RecordingConductor {
    received: Rc<RefCell<Vec<(usize, MidiMessage)>>>,
}

impl Conductor for RecordingConductor {
    fn init(&mut self, _context: &mut Context) -> Vec<Instruction> {
        vec![]
    }

    fn update(&mut self, _context: &mut Context) -> Vec<Instruction> {
        vec![]
    }

    fn handle_input(
        &mut self,
        input_id: usize,
        input: MidiMessage,
        _context: &Context,
    ) -> InputResponse {
        self.received.borrow_mut().push((input_id, input));
        InputResponse::default()
    }
}

/// Mirrors the routing performed by the midir input callback in `mseq::connect`:
/// transport messages (Clock/Start/Stop/Continue) are forwarded to the system queue
/// only for the slave input and dropped for any other input; channel messages always
/// go to the input's own message queue.
fn route(
    msg: MidiMessage,
    is_slave: bool,
    system_queue: &mut VecDeque<MidiMessage>,
    message_queue: &mut InputQueue,
) {
    if msg.is_transport() {
        if is_slave {
            system_queue.push_back(msg);
        }
        // Dropped for non-slave inputs.
    } else {
        message_queue.push_back(msg);
    }
}

/// Mirrors the per-clock stepping of `mseq::run_slave`: the sequencer advances exactly
/// one step for every Clock message read from the slave system queue, after applying any
/// Start/Stop/Continue that arrived before it.
fn run_slave_sim(
    ctx: &mut Context,
    conductor: &mut impl Conductor,
    controller: &mut MidiController<impl MidiOut>,
    system_queue: VecDeque<MidiMessage>,
) {
    let mut pending_start = false;
    for m in system_queue {
        match m {
            MidiMessage::Clock => {
                ctx.process_pre_tick(conductor, controller);
                if pending_start {
                    ctx.start();
                    pending_start = false;
                }
                ctx.process_post_tick(controller);
            }
            MidiMessage::Start => pending_start = true,
            MidiMessage::Stop => ctx.pause(),
            MidiMessage::Continue => ctx.resume(),
            _ => unreachable!("only transport messages reach the system queue"),
        }
    }
}

#[test]
fn test_slave_multi_input() {
    let debug_conn = Rc::new(RefCell::new(DebugMidiOutInner {
        notes_on: HashMap::new(),
        start_timestamp: Instant::now(),
    }));
    let mut controller = MidiController::new(DebugMidiOut(debug_conn));

    let received = Rc::new(RefCell::new(vec![]));
    let mut conductor = RecordingConductor {
        received: received.clone(),
    };

    let mut ctx = Context::default();
    conductor.init(&mut ctx);
    // Slave mode starts paused, like `mseq::run_slave`.
    ctx.pause();

    // One system queue for the slave input and one message queue per input.
    let mut system_queue: VecDeque<MidiMessage> = VecDeque::new();
    let mut message_queues: Vec<InputQueue> = (0..NUM_INPUTS).map(|_| InputQueue::new()).collect();

    const CLOCKS_INPUT_0: usize = 5;
    const CLOCKS_INPUT_1: usize = 3;

    // Input 0 (the slave): Start then a stream of clocks. These must drive the step.
    route(
        MidiMessage::Start,
        true,
        &mut system_queue,
        &mut message_queues[SLAVE_INPUT],
    );
    for _ in 0..CLOCKS_INPUT_0 {
        route(
            MidiMessage::Clock,
            true,
            &mut system_queue,
            &mut message_queues[SLAVE_INPUT],
        );
    }

    // Input 1 (non-slave): clocks (which must be dropped and never advance the step)
    // plus one channel message (which must still reach the conductor).
    let note_on = MidiMessage::NoteOn {
        channel: 1,
        note: MidiNote::new(Note::C, 4, 100),
    };
    for _ in 0..CLOCKS_INPUT_1 {
        route(
            MidiMessage::Clock,
            false,
            &mut system_queue,
            &mut message_queues[1],
        );
    }
    route(note_on, false, &mut system_queue, &mut message_queues[1]);

    // Input 1's clocks were dropped at routing: only its channel message remains.
    assert_eq!(message_queues[1].len(), 1);

    // Drive the slave loop from input 0's system queue.
    run_slave_sim(&mut ctx, &mut conductor, &mut controller, system_queue);

    // Only input 0's clocks advanced the step.
    assert_eq!(ctx.get_step(), CLOCKS_INPUT_0 as u32);

    // Now deliver every input's channel messages, as the per-input consumer threads do.
    for (input_id, queue) in message_queues.iter_mut().enumerate() {
        ctx.handle_input(input_id, &mut conductor, &mut controller, queue);
    }

    // The conductor only ever saw input 1's channel message: no transport message
    // (from any input) reached `handle_input`, and the step is unchanged.
    assert_eq!(*received.borrow(), vec![(1, note_on)]);
    assert_eq!(ctx.get_step(), CLOCKS_INPUT_0 as u32);
}
