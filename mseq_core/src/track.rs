use crate::MidiNote;
use crate::midi_controller::Instruction;
use crate::note::Note;
use serde::{Deserialize, Serialize};

use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

/// Abstraction for a sequencer track.
///
/// Users can create their own custom track implementations by implementing this trait,
/// defining how steps are played and optionally providing a track name.
pub trait Track {
    /// Plays the given step in the track.
    ///
    /// # Returns
    ///
    /// A vector of `Instruction` produced by the track at this step.
    fn play_step(&mut self, step: u32) -> Vec<Instruction>;
    /// Returns the name of the track.
    fn get_name(&self) -> String {
        "Unamed".to_string()
    }
}

/// A deterministic track implementation.
///
/// `DeteTrack` implements the [`Track`] trait by playing a fixed pattern
/// in a continuous loop. Each call to `play_step` produces the same
/// sequence of instructions based on the step index modulo the pattern length.
#[derive(Default, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DeteTrack {
    len: u32,
    notes: Vec<(MidiNote, u32, u32)>, // (Note, start step, length)
    start_step: u32,
    root: Note,
    transpose: Option<i8>,
    channel_id: u8,
    name: String,
}

impl Track for DeteTrack {
    fn play_step(&mut self, step: u32) -> Vec<Instruction> {
        let cur_step = step % self.len;
        self.notes
            .iter()
            .filter_map(|n| {
                if (n.1 + self.start_step) % self.len == cur_step {
                    let note = self.transpose.map_or(n.0, |t| n.0.transpose(t));

                    Some(Instruction::PlayNote {
                        midi_note: note,
                        len: n.2,
                        channel_id: self.channel_id,
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }
}

impl DeteTrack {
    /// Creates a new DeteTrack from a list of notes, its length, the midi channel (1-16) and a name.
    /// Specify the root note to allow transposition.
    pub fn new(
        len: u32,
        notes: Vec<(MidiNote, u32, u32)>,
        root: Note,
        channel_id: u8,
        name: &str,
    ) -> Self {
        DeteTrack {
            len,
            notes,
            start_step: 0,
            root,
            transpose: None,
            channel_id,
            name: name.to_string(),
        }
    }

    /// Sets the root of the DeteTrack. This function does not transpose the track, it only changes
    /// the root note.
    pub fn set_root(&mut self, note: Note) {
        self.root = note;
    }

    /// Returns the all `(note, length)`, that start at `step`. Transposition and start step are
    /// taken into account.
    pub fn get_notes_start_at_step(&self, step: u32) -> Vec<(MidiNote, u32)> {
        let mut notes = vec![];
        let cur_step = step % self.len;
        for n in &self.notes {
            if (n.1 + self.start_step) % self.len == cur_step {
                let note = self.transpose.map_or(n.0, |t| n.0.transpose(t));
                notes.push((note, n.2));
            }
        }
        notes
    }

    /// Transposes the DeteTrack.
    pub fn transpose(&mut self, note: Option<Note>) {
        self.transpose = note.map(|n| Note::transpose(self.root, n));
    }
}
