use crate::MidiNote;
use std::{path::Path, u32};

use crate::{
    midi_controller::MidiController,
    note::{self, Note},
};

pub trait Track {
    fn play_step(&mut self, step: u32, midi_controller: &mut MidiController);
    fn transpose(&mut self, note: Option<Note>) {}
    fn get_root(&self) -> Note {
        Note::C
    }
}

struct DeteTrack {
    len: u32,
    notes: Vec<(MidiNote, u32, u32)>, // (Note, start step, length)
    start_step: u32,
    root: Note,
    transpose: Option<i8>,
    channel_id: u8,
}

impl Track for DeteTrack {
    fn play_step(&mut self, step: u32, midi_controller: &mut MidiController) {
        let cur_step = step % self.len;
        for n in &self.notes {
            if (n.1 + self.start_step) % self.len == cur_step {
                match self.transpose {
                    Some(t) => midi_controller.play_note(n.0.transpose(t), n.2, self.channel_id),
                    _ => midi_controller.play_note(n.0, n.2, self.channel_id),
                }
            }
        }
    }

    fn transpose(&mut self, note: Option<Note>) {
        self.transpose = match note {
            Some(n) => Some(Note::transpose(self.root, n)),
            _ => None,
        };
    }

    fn get_root(&self) -> Note {
        self.root
    }
}

impl DeteTrack {
    pub fn set_start_step(&mut self, start_step: u32) {
        self.start_step = start_step;
    }

    pub fn set_root(&mut self, note: Note) {
        self.root = note;
    }

    pub fn load_from_file<P: AsRef<Path>>(filename: P) {}
}
