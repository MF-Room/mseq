use crate::note::Note;
use crate::{log_send, note};
use midir::MidiOutputConnection;
use std::collections::HashMap;

#[derive(Default, Clone, Copy)]
pub struct MidiNote {
    note: Note,
    octave: u8,
    vel: u8,
}

impl MidiNote {
    pub fn new(note: Note, octave: u8, vel: u8) -> Self {
        Self { note, octave, vel }
    }
}

#[derive(Default, Clone, Copy)]
struct NotePlay {
    note: MidiNote,
    channel_id: u8,
}

impl NotePlay {
    fn new(note: MidiNote, channel_id: u8) -> Self {
        Self { note, channel_id }
    }
}

pub struct MidiController {
    note_off: HashMap<u32, Vec<NotePlay>>,
    note_on: Vec<NotePlay>,
    pub conn: MidiOutputConnection,
    step: u32,
}

impl MidiController {
    pub fn new(conn: MidiOutputConnection) -> Self {
        Self {
            note_off: HashMap::new(),
            note_on: vec![],
            conn,
            step: 0,
        }
    }

    pub fn play_note(&mut self, note: MidiNote, len: u32, channel_id: u8) {
        let note_play = NotePlay::new(note, channel_id);
        self.note_on.push(note_play);
        let step = self.step + len;
        let notes = self.note_off.get_mut(&(step));
        match notes {
            Some(n) => n.push(note_play),
            _ => {
                let n = vec![note_play];
                self.note_off.insert(step, n);
            }
        }
    }

    pub fn update(&mut self, step: u32) {
        self.step = step;
        let notes = self.note_off.remove(&step);
        for note in &self.note_on {
            log_send(
                &mut self.conn,
                &start_note(
                    note.channel_id,
                    note.note.note.get_midi() + 12 * note.note.octave,
                    note.note.vel,
                ),
            );
        }
        self.note_on.clear();
        match notes {
            Some(notes) => {
                for n in notes {
                    log_send(
                        &mut self.conn,
                        &end_note(
                            n.channel_id,
                            n.note.note.get_midi() + 12 * n.note.octave,
                            n.note.vel,
                        ),
                    );
                }
            }
            _ => {}
        }
    }
}

pub fn cc_parameter(parameter: u8, sp: u8) -> u8 {
    parameter + 10 * (sp + 1)
}

pub const NOTE_ON: u8 = 0x90;
pub const NOTE_OFF: u8 = 0x80;
pub const CC: u8 = 0xB0;

fn start_note(channel_id: u8, note: u8, velocity: u8) -> Vec<u8> {
    vec![NOTE_ON | channel_id, note, velocity]
}

fn end_note(channel_id: u8, note: u8, velocity: u8) -> Vec<u8> {
    vec![NOTE_OFF | channel_id, note, velocity]
}

pub fn control_change(channel_id: u8, parameter: u8, value: u8) -> Vec<u8> {
    vec![CC | channel_id, parameter, value]
}

pub fn param_value(v: f32) -> u8 {
    if v < -1.0 {
        return 0;
    }
    if v > 1.0 {
        return 127;
    }
    63 + (v * 63.0).round() as u8
}
