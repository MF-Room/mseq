use std::path::Path;

use crate::{midi_controller::MidiController, note::Note};

trait Track {
    fn play_step(&mut self, step: u32, midi_controller: &mut MidiController);
    fn transpose(note: Note) {
    }
    fn get_root() ->  Note {
        Note::C
    }
}

struct MidiTrack {
}

impl Track for MidiTrack {
    fn play_step(&mut self, step: u32, midi_controller: &mut MidiController) {
        todo!()
    }
}

impl MidiTrack {
    fn load_from_file<P: AsRef<Path>>(filename: P) {

    }
}
