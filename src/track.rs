use crate::{MidiConnection, MidiNote};
use std::path::Path;

use crate::{midi_controller::MidiController, note::Note};

pub trait Track {
    fn play_step<T: MidiConnection>(&mut self, step: u32, midi_controller: &mut MidiController<T>);
    fn transpose(&mut self, _note: Option<Note>) {
        // Todo: add warning
    }
    fn get_root(&self) -> Note {
        Note::C
    }
    fn set_start_step(&mut self, _start_step: u32) {
        // Todo: add warning
    }

    fn get_name(&self) -> String {
        "Unamed".to_string()
    }
}

#[derive(Default, Clone)]
pub struct DeteTrack {
    len: u32,
    notes: Vec<(MidiNote, u32, u32)>, // (Note, start step, length)
    pub start_step: u32,
    root: Note,
    transpose: Option<i8>,
    pub channel_id: u8,
    name: String,
}

impl Track for DeteTrack {
    fn play_step<T: MidiConnection>(&mut self, step: u32, midi_controller: &mut MidiController<T>) {
        let cur_step = step % self.len;
        for n in &self.notes {
            if (n.1 + self.start_step) % self.len == cur_step {
                let note = self.transpose.map_or(n.0, |t| n.0.transpose(t));
                midi_controller.play_note(note, n.2, self.channel_id)
            }
        }
    }

    fn transpose(&mut self, note: Option<Note>) {
        self.transpose = note.map(|n| Note::transpose(self.root, n));
    }

    fn get_root(&self) -> Note {
        self.root
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }
}

impl DeteTrack {
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

    pub fn set_root(&mut self, note: Note) {
        self.root = note;
    }

    pub fn load_from_file<P: AsRef<Path>>(_filename: P) -> Self {
        todo!()
    }
}
