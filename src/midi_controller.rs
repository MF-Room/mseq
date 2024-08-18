use crate::note::Note;
use crate::{log_send, Track};
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

    pub fn transpose(&self, transpose: i8) -> Self {
        let (note, octave) = if transpose >= 0 {
            (self.note.add_semitone(transpose as u8), self.octave)
        } else {
            (
                self.note.add_semitone((12 - transpose) as u8),
                self.octave - 1,
            )
        };
        Self {
            note,
            octave,
            vel: self.vel,
        }
    }
}

#[derive(Default, Clone, Copy)]
struct NotePlay {
    midi_note: MidiNote,
    channel_id: u8,
}

impl NotePlay {
    fn new(midi_note: MidiNote, channel_id: u8) -> Self {
        Self {
            midi_note,
            channel_id,
        }
    }
}

pub struct MidiController {
    notes_off: HashMap<u32, Vec<NotePlay>>,
    notes_on: Vec<NotePlay>,
    pub conn: MidiOutputConnection,
    step: u32,
}

impl MidiController {
    pub fn new(conn: MidiOutputConnection) -> Self {
        Self {
            notes_off: HashMap::new(),
            notes_on: vec![],
            conn,
            step: 0,
        }
    }

    pub fn play_note(&mut self, midi_note: MidiNote, len: u32, channel_id: u8) {
        let note_play = NotePlay::new(midi_note, channel_id);
        self.notes_on.push(note_play);
        let step = self.step + len;
        let notes = self.notes_off.get_mut(&(step));
        match notes {
            Some(n) => n.push(note_play),
            _ => {
                let n = vec![note_play];
                self.notes_off.insert(step, n);
            }
        }
    }

    pub fn update(&mut self, step: u32) {
        self.step = step;

        for note_on in &self.notes_on {
            log_send(
                &mut self.conn,
                &start_note(
                    note_on.channel_id,
                    u8::from(note_on.midi_note.note) + 12 * note_on.midi_note.octave,
                    note_on.midi_note.vel,
                ),
            );
        }
        self.notes_on.clear();

        let notes = self.notes_off.remove(&step);
        if let Some(notes_off) = notes {
            for n in notes_off {
                log_send(
                    &mut self.conn,
                    &end_note(
                        n.channel_id,
                        u8::from(n.midi_note.note) + 12 * n.midi_note.octave,
                        n.midi_note.vel,
                    ),
                );
            }
        };
    }

    pub fn play_track(&mut self, track: &mut impl Track) {
        track.play_step(self.step, self);
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
