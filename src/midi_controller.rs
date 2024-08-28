use crate::note::Note;
use crate::Track;
use midir::MidiOutputConnection;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

const MAX_MIDI_CHANNEL: u8 = 16;

const CLOCK_MIDI: u8 = 0xf8;
const START_MIDI: u8 = 0xfa;
const CONTINUE_MIDI: u8 = 0xfb;
const STOP_MIDI: u8 = 0xfc;
const NOTE_ON: u8 = 0x90;
const NOTE_OFF: u8 = 0x80;
const CC: u8 = 0xB0;

#[derive(Default, Clone, Copy, Debug, serde::Deserialize, PartialEq, Eq)]
pub struct MidiNote {
    pub note: Note,
    pub octave: u8,
    pub vel: u8,
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

    pub fn midi_value(&self) -> u8 {
        u8::from(self.note) + 12 * self.octave
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
struct NotePlay {
    midi_note: MidiNote,
    channel_id: u8,
}

impl Hash for NotePlay {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (self.midi_note.midi_value() as u32 + MAX_MIDI_CHANNEL as u32 * self.channel_id as u32)
            .hash(state);
    }
}

pub struct MidiController {
    notes_off: HashMap<u32, Vec<NotePlay>>,
    notes_on: HashSet<NotePlay>,
    pub conn: MidiOutputConnection,
    step: u32,
}

impl MidiController {
    pub(crate) fn new(conn: MidiOutputConnection) -> Self {
        Self {
            notes_off: HashMap::new(),
            notes_on: HashSet::new(),
            conn,
            step: 0,
        }
    }

    pub fn play_track(&mut self, track: &mut impl Track) {
        track.play_step(self.step, self);
    }

    pub fn play_note(&mut self, midi_note: MidiNote, len: u32, channel_id: u8) {
        if len == 0 {
            return;
        }

        self.start_note(midi_note, channel_id);
        let note_play = NotePlay {
            midi_note,
            channel_id,
        };

        let stop_step = self.step + len;
        self.stop_note_at_step(note_play, stop_step);
    }

    pub fn start_note(&mut self, midi_note: MidiNote, channel_id: u8) {
        let note_play = NotePlay {
            midi_note,
            channel_id,
        };
        self.notes_on.insert(note_play);
    }

    pub fn stop_note(&mut self, midi_note: MidiNote, channel_id: u8) {
        let note_play = NotePlay {
            midi_note,
            channel_id,
        };
        self.stop_note_at_step(note_play, self.step + 1);
    }

    fn stop_note_at_step(&mut self, note_play: NotePlay, step: u32) {
        if self.notes_on.remove(&note_play) {
            self.notes_off.entry(step).or_insert(vec![]).push(note_play);
        }
    }

    pub fn send_cc(&mut self, channel_id: u8, parameter: u8, value: u8) {
        let message = vec![CC | channel_id, parameter, value];
        log_send(&mut self.conn, &message);
    }

    pub(crate) fn send_clock(&mut self) {
        log_send(&mut self.conn, &[CLOCK_MIDI]);
    }

    pub(crate) fn start(&mut self) {
        self.step = 0;
        log_send(&mut self.conn, &[START_MIDI]);
    }

    pub(crate) fn send_continue(&mut self) {
        log_send(&mut self.conn, &[CONTINUE_MIDI]);
    }

    pub(crate) fn update(&mut self, next_step: u32) {
        let notes = self.notes_off.remove(&self.step);
        if let Some(notes_off) = notes {
            for n in notes_off {
                log_send(
                    &mut self.conn,
                    &end_note(n.channel_id, n.midi_note.midi_value(), n.midi_note.vel),
                );
            }
        };

        for n in &self.notes_on {
            log_send(
                &mut self.conn,
                &start_note(n.channel_id, n.midi_note.midi_value(), n.midi_note.vel),
            );
        }
        self.notes_on.clear();

        self.step = next_step;
    }

    pub(crate) fn stop(&mut self) {
        self.notes_off.values().flatten().for_each(|n| {
            log_send(
                &mut self.conn,
                &end_note(n.channel_id, n.midi_note.midi_value(), n.midi_note.vel),
            );
        });

        self.notes_on.iter().for_each(|n| {
            log_send(
                &mut self.conn,
                &end_note(n.channel_id, n.midi_note.midi_value(), n.midi_note.vel),
            );
        });

        self.notes_off.clear();
    }

    pub(crate) fn pause(&mut self) {
        log_send(&mut self.conn, &[STOP_MIDI]);
    }
}

fn start_note(channel_id: u8, note: u8, velocity: u8) -> Vec<u8> {
    vec![NOTE_ON | channel_id, note, velocity]
}

fn end_note(channel_id: u8, note: u8, velocity: u8) -> Vec<u8> {
    vec![NOTE_OFF | channel_id, note, velocity]
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

fn log_send(conn: &mut MidiOutputConnection, message: &[u8]) {
    if let Err(x) = conn.send(message) {
        eprintln!("[ERROR] {} (message: {:?})", x, message)
    }
}
