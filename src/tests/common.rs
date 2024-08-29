use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::clock::Clock;
use crate::Conductor;
use crate::Context;
use crate::MidiConnection;
use crate::MidiController;
use crate::MidiError;

pub(super) struct DebugMidiConnectionInner {
    pub notes_on: HashMap<(u8, u8), u8>,
}

pub(super) struct DebugMidiConnection(pub Rc<RefCell<DebugMidiConnectionInner>>);

impl MidiConnection for DebugMidiConnection {
    fn send_start(&mut self) -> Result<(), MidiError> {
        println!("start");
        Ok(())
    }

    fn send_continue(&mut self) -> Result<(), MidiError> {
        println!("continue");
        Ok(())
    }

    fn send_stop(&mut self) -> Result<(), MidiError> {
        println!("stop");
        Ok(())
    }

    fn send_clock(&mut self) -> Result<(), MidiError> {
        println!("clock");
        Ok(())
    }

    fn send_note_on(&mut self, channel_id: u8, note: u8, velocity: u8) -> Result<(), MidiError> {
        println!(
            "sending note on: channel {}, note {}, vel {}",
            channel_id, note, velocity
        );
        self.0
            .borrow_mut()
            .notes_on
            .insert((channel_id, note), velocity);
        Ok(())
    }

    fn send_note_off(&mut self, channel_id: u8, note: u8) -> Result<(), MidiError> {
        println!("sending note on: channel {}, note {}", channel_id, note);
        let mut inner = self.0.borrow_mut();
        assert!(inner.notes_on.contains_key(&(channel_id, note)));
        inner.notes_on.remove(&(channel_id, note));
        Ok(())
    }

    fn send_cc(&mut self, _channel_id: u8, _parameter: u8, _value: u8) -> Result<(), MidiError> {
        Ok(())
    }
}

pub(super) fn test_conductor(mut conductor: impl Conductor, midi: MidiController) {
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
