use crate::midi_connection::MidiConnection;
use crate::note::Note;
use crate::Track;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

const MAX_MIDI_CHANNEL: u8 = 16;

#[derive(Default, Clone, Copy, serde::Deserialize, PartialEq, Eq, Debug)]
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

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
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
    /// Current midi step
    step: u32,

    /// Every note currently being played triggered by play_note. The key is the step at which to stop the note.
    notes_off: HashMap<u32, Vec<NotePlay>>,

    /// Every note currently being played triggered by start_note.
    notes_on: HashSet<NotePlay>,

    /// Notes to play at the next update call
    notes_to_play: Vec<NotePlay>,

    conn: Box<dyn MidiConnection>,
}

impl MidiController {
    pub(crate) fn new(conn: Box<dyn MidiConnection>) -> Self {
        Self {
            step: 0,
            notes_off: HashMap::new(),
            notes_on: HashSet::new(),
            notes_to_play: vec![],
            conn,
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
        self.notes_to_play.push(note_play);
        self.notes_on.insert(note_play);
    }

    pub fn stop_note(&mut self, midi_note: MidiNote, channel_id: u8) {
        let note_play = NotePlay {
            midi_note,
            channel_id,
        };
        self.stop_note_at_step(note_play, self.step);
    }

    fn stop_note_at_step(&mut self, note_play: NotePlay, step: u32) {
        if self.notes_on.remove(&note_play) {
            self.notes_off.entry(step).or_default().push(note_play);
        }
    }

    pub fn send_cc(&mut self, channel_id: u8, parameter: u8, value: u8) {
        if let Err(_e) = self.conn.send_cc(channel_id, parameter, value) {
            crate::log_error!("Midi Error: {:?}", _e);
        }
    }

    pub(crate) fn send_clock(&mut self) {
        if let Err(_e) = self.conn.send_clock() {
            crate::log_error!("Midi Error: {:?}", _e);
        }
    }

    pub(crate) fn start(&mut self) {
        self.step = 0;
        if let Err(_e) = self.conn.send_start() {
            crate::log_error!("Midi Error: {:?}", _e);
        }
    }

    pub(crate) fn send_continue(&mut self) {
        if let Err(_e) = self.conn.send_continue() {
            crate::log_error!("Midi Error: {:?}", _e);
        }
    }

    pub(crate) fn update(&mut self, next_step: u32) {
        // First send the off signal to every note that end this step.
        let notes = self.notes_off.remove(&self.step);
        if let Some(notes_off) = notes {
            for n in notes_off {
                if let Err(_e) = self
                    .conn
                    .send_note_off(n.channel_id, n.midi_note.midi_value())
                {
                    crate::log_error!("Midi Error: {:?}", _e);
                }
            }
        };

        // Then play all the notes that were triggered this step...
        for n in &self.notes_to_play {
            if let Err(_e) =
                self.conn
                    .send_note_on(n.channel_id, n.midi_note.midi_value(), n.midi_note.vel)
            {
                crate::log_error!("Midi Error: {:?}", _e);
            }
        }
        // ...and clear them.
        self.notes_to_play.clear();

        // Finally update the step.
        self.step = next_step;
    }

    pub(crate) fn stop_all_notes(&mut self) {
        self.notes_on.iter().for_each(|n| {
            if let Err(_e) = self
                .conn
                .send_note_off(n.channel_id, n.midi_note.midi_value())
            {
                crate::log_error!("Midi Error: {:?}", _e);
            }
        });
        self.notes_on.clear();

        self.notes_off.iter().for_each(|(_, notes)| {
            for n in notes {
                if let Err(_e) = self
                    .conn
                    .send_note_off(n.channel_id, n.midi_note.midi_value())
                {
                    crate::log_error!("Midi Error: {:?}", _e);
                }
            }
        });
        self.notes_off.clear();
    }

    pub(crate) fn stop(&mut self) {
        if let Err(_e) = self.conn.send_stop() {
            crate::log_error!("Midi Error: {:?}", _e);
        }
    }
}
