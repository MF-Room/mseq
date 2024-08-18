#[derive(Debug, Default, Clone, Copy)]
pub enum Note {
    #[default]
    C,
    CS,
    D,
    DS,
    E,
    F,
    FS,
    G,
    GS,
    A,
    AS,
    B,
}

impl Note {
    pub fn add_semitone(&self, semi: u8) -> Self {
        Self::get_note(self.get_midi() + semi)
    }

    pub fn transpose(root: Note, note: Note) -> i8 {
        let root_m = root.get_midi() as i8;
        let note_m = note.get_midi() as i8;
        let n = (note_m - root_m) % 12;
        if n > 6 {
            n - 12
        } else {
            n
        }
    }

    pub fn get_midi(&self) -> u8 {
        match self {
            Note::C => 12,
            Note::CS => 13,
            Note::D => 14,
            Note::DS => 15,
            Note::E => 16,
            Note::F => 17,
            Note::FS => 18,
            Note::G => 19,
            Note::GS => 20,
            Note::A => 21,
            Note::AS => 22,
            Note::B => 23,
        }
    }

    pub fn get_note(note: u8) -> Self {
        let n = note % 12;
        match n {
            0 => Note::C,
            1 => Note::CS,
            2 => Note::D,
            3 => Note::DS,
            4 => Note::E,
            5 => Note::F,
            6 => Note::FS,
            7 => Note::G,
            8 => Note::GS,
            9 => Note::A,
            10 => Note::AS,
            _ => Note::B,
        }
    }

    pub fn get_str(&self) -> &str {
        match self {
            Note::C => "C",
            Note::CS => "C#",
            Note::D => "D",
            Note::DS => "D#",
            Note::E => "E",
            Note::F => "F",
            Note::FS => "F#",
            Note::G => "G",
            Note::GS => "G#",
            Note::A => "A",
            Note::AS => "A#",
            Note::B => "B",
        }
    }
}
