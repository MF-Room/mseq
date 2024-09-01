use crate::clock::Clock;
use crate::Conductor;
use crate::Context;
use crate::MidiConnection;
use crate::MidiController;
use crate::MidiError;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Instant;

pub(super) struct DebugMidiConnectionInner {
    pub notes_on: HashMap<(u8, u8), u8>,
    pub start_timestamp: Instant,
}

pub(super) struct DebugMidiConnection(pub Rc<RefCell<DebugMidiConnectionInner>>);

impl DebugMidiConnection {
    fn print_elapsed(&self, message: &str) {
        println!(
            "[{}]\t{message}",
            self.0.borrow().start_timestamp.elapsed().as_secs_f32()
        )
    }
}
impl MidiConnection for DebugMidiConnection {
    fn send_start(&mut self) -> Result<(), MidiError> {
        self.print_elapsed("Start");
        Ok(())
    }

    fn send_continue(&mut self) -> Result<(), MidiError> {
        self.print_elapsed("Continue");
        Ok(())
    }

    fn send_stop(&mut self) -> Result<(), MidiError> {
        self.print_elapsed("Stop");
        Ok(())
    }

    fn send_clock(&mut self) -> Result<(), MidiError> {
        self.print_elapsed("Clock");
        Ok(())
    }

    fn send_note_on(&mut self, channel_id: u8, note: u8, velocity: u8) -> Result<(), MidiError> {
        let message = format!("On\tchn:{}\tnte:{}\tvel:{}", channel_id, note, velocity);
        self.print_elapsed(&message);
        self.0
            .borrow_mut()
            .notes_on
            .insert((channel_id, note), velocity);
        Ok(())
    }

    fn send_note_off(&mut self, channel_id: u8, note: u8) -> Result<(), MidiError> {
        let message = format!("Off\tchn:{}\tnte:{}", channel_id, note);
        self.print_elapsed(&message);
        let mut inner = self.0.borrow_mut();
        assert!(inner.notes_on.contains_key(&(channel_id, note)));
        inner.notes_on.remove(&(channel_id, note));
        Ok(())
    }

    fn send_cc(&mut self, channel_id: u8, parameter: u8, value: u8) -> Result<(), MidiError> {
        let message = format!("Cc\tchn:{}\tprm:{}\tval:{}", channel_id, parameter, value);
        self.print_elapsed(&message);
        Ok(())
    }
}

pub(super) fn test_conductor<T: MidiConnection>(
    mut conductor: impl Conductor,
    midi: MidiController<T>,
) {
    let mut ctx = Context {
        midi,
        clock: Clock::new(120),
        step: 0,
        running: true,
        on_pause: false,
        pause: false,
    };
    conductor.init(&mut ctx);
    ctx.run(conductor);
}
