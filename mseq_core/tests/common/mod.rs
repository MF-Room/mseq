use mseq_core::Conductor;
use mseq_core::Context;
use mseq_core::MidiController;
use mseq_core::MidiOut;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Instant;

pub struct DebugMidiOutInner {
    pub notes_on: HashMap<(u8, u8), u8>,
    pub start_timestamp: Instant,
}

pub struct DebugMidiOut(pub Rc<RefCell<DebugMidiOutInner>>);

impl DebugMidiOut {
    fn print_elapsed(&self, message: &str) {
        println!(
            "[{}]\t{message}",
            self.0.borrow().start_timestamp.elapsed().as_secs_f32()
        )
    }
}
impl MidiOut for DebugMidiOut {
    type Error = String;
    fn send_start(&mut self) -> Result<(), String> {
        self.print_elapsed("Start");
        Ok(())
    }

    fn send_continue(&mut self) -> Result<(), String> {
        self.print_elapsed("Continue");
        Ok(())
    }

    fn send_stop(&mut self) -> Result<(), String> {
        self.print_elapsed("Stop");
        Ok(())
    }

    fn send_clock(&mut self) -> Result<(), String> {
        self.print_elapsed("Clock");
        Ok(())
    }

    fn send_note_on(&mut self, channel_id: u8, note: u8, velocity: u8) -> Result<(), String> {
        let message = format!("On\tchn:{}\tnte:{}\tvel:{}", channel_id, note, velocity);
        self.print_elapsed(&message);
        self.0
            .borrow_mut()
            .notes_on
            .insert((channel_id, note), velocity);
        Ok(())
    }

    fn send_note_off(&mut self, channel_id: u8, note: u8) -> Result<(), String> {
        let message = format!("Off\tchn:{}\tnte:{}", channel_id, note);
        self.print_elapsed(&message);
        let mut inner = self.0.borrow_mut();
        assert!(inner.notes_on.contains_key(&(channel_id, note)));
        inner.notes_on.remove(&(channel_id, note));
        Ok(())
    }

    fn send_cc(&mut self, channel_id: u8, parameter: u8, value: u8) -> Result<(), String> {
        let message = format!("Cc\tchn:{}\tprm:{}\tval:{}", channel_id, parameter, value);
        self.print_elapsed(&message);
        Ok(())
    }
}

pub fn test_conductor<T: MidiOut>(
    mut conductor: impl Conductor,
    mut midi_controller: MidiController<T>,
) {
    let mut ctx = Context::new();
    conductor.init(&mut ctx);
    while ctx.is_running() {
        ctx.process_pre_tick(&mut conductor, &mut midi_controller);
        // No sleep between each cycle because we're testing
        ctx.process_post_tick(&mut midi_controller);
    }
    midi_controller.stop_all_notes(None);
    midi_controller.stop();
}
