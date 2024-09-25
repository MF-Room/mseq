use crate::midi_connection::MidiConnection;
use crate::note::Note;
use crate::Track;
use log::error;
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

    pub fn from_midi_value(midi_value: u8, vel: u8) -> Self {
        let octave = midi_value / 12;
        let note = Note::from(midi_value % 12);
        Self::new(note, octave, vel)
    }

    pub fn transpose(&self, transpose: i8) -> Self {
        let (note, octave) = self.note.add_semitone(self.octave, transpose);
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

pub struct MidiController<T: MidiConnection> {
    /// Current midi step
    step: u32,

    /// Every note currently being played triggered by play_note. The key is the step at which to stop the note.
    play_note_set: HashMap<u32, Vec<NotePlay>>,

    /// Every note currently being played triggered by start_note.
    start_note_set: HashSet<NotePlay>,

    /// Notes to play at the next update call
    notes_to_play: Vec<NotePlay>,

    conn: T,
}

impl<T: MidiConnection> MidiController<T> {
    pub(crate) fn new(conn: T) -> Self {
        Self {
            step: 0,
            play_note_set: HashMap::new(),
            start_note_set: HashSet::new(),
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

        let note_play = NotePlay {
            midi_note,
            channel_id,
        };
        self.notes_to_play.push(note_play);
        self.stop_note_at_step(note_play, self.step + len);
    }

    pub fn start_note(&mut self, midi_note: MidiNote, channel_id: u8) {
        let note_play = NotePlay {
            midi_note,
            channel_id,
        };
        self.notes_to_play.push(note_play);
        self.start_note_set.insert(note_play);
    }

    pub fn stop_note(&mut self, midi_note: MidiNote, channel_id: u8) {
        let note_play = NotePlay {
            midi_note,
            channel_id,
        };
        self.stop_note_at_step(note_play, self.step);
    }

    fn stop_note_at_step(&mut self, note_play: NotePlay, step: u32) {
        self.play_note_set.entry(step).or_default().push(note_play);
    }

    pub fn send_cc(&mut self, channel_id: u8, parameter: u8, value: u8) {
        if let Err(e) = self.conn.send_cc(channel_id, parameter, value) {
            error!("MIDI: {e}");
        }
    }

    pub(crate) fn send_clock(&mut self) {
        if let Err(e) = self.conn.send_clock() {
            error!("MIDI: {e}");
        }
    }

    pub(crate) fn start(&mut self) {
        self.step = 0;
        if let Err(e) = self.conn.send_start() {
            error!("MIDI: {e}");
        }
    }

    pub(crate) fn send_continue(&mut self) {
        if let Err(e) = self.conn.send_continue() {
            error!("MIDI: {e}");
        }
    }

    pub(crate) fn update(&mut self, next_step: u32) {
        // First send the off signal to every note that end this step.
        let notes = self.play_note_set.remove(&self.step);
        if let Some(notes_off) = notes {
            for n in notes_off {
                if let Err(e) = self
                    .conn
                    .send_note_off(n.channel_id, n.midi_note.midi_value())
                {
                    error!("MIDI: {e}");
                }
            }
        };

        // Then play all the notes that were triggered this step...
        for n in &self.notes_to_play {
            if let Err(e) =
                self.conn
                    .send_note_on(n.channel_id, n.midi_note.midi_value(), n.midi_note.vel)
            {
                error!("MIDI: {e}");
            }
        }
        // ...and clear them.
        self.notes_to_play.clear();

        // Finally update the step.
        self.step = next_step;
    }

    pub(crate) fn stop_all_notes(&mut self) {
        self.start_note_set.iter().for_each(|n| {
            if let Err(e) = self
                .conn
                .send_note_off(n.channel_id, n.midi_note.midi_value())
            {
                error!("MIDI: {e}");
            }
        });
        self.start_note_set.clear();

        self.play_note_set.values().for_each(|notes| {
            for n in notes {
                if let Err(e) = self
                    .conn
                    .send_note_off(n.channel_id, n.midi_note.midi_value())
                {
                    error!("MIDI: {e}");
                }
            }
        });
        self.play_note_set.clear();
    }

    pub(crate) fn stop(&mut self) {
        if let Err(e) = self.conn.send_stop() {
            error!("MIDI: {e}");
        }
    }
}
