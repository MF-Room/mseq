use std::{convert::From, fmt::Display};

#[derive(Debug, Default, Clone, PartialEq, Copy, serde::Deserialize, Eq)]
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

impl From<Note> for u8 {
    fn from(note: Note) -> Self {
        match note {
            Note::C => 0,
            Note::CS => 1,
            Note::D => 2,
            Note::DS => 3,
            Note::E => 4,
            Note::F => 5,
            Note::FS => 6,
            Note::G => 7,
            Note::GS => 8,
            Note::A => 9,
            Note::AS => 10,
            Note::B => 11,
        }
    }
}

impl From<u8> for Note {
    fn from(midi: u8) -> Self {
        let n = midi % 12;
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
            11 => Note::B,
            _ => unreachable!(),
        }
    }
}

impl Display for Note {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
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
        };
        write!(f, "{}", str)
    }
}

impl Note {
    pub fn add_semitone(self, semi: u8) -> Self {
        (u8::from(self) + semi).into()
    }

    pub fn transpose(root: Note, note: Note) -> i8 {
        let root_m: u8 = root.into();
        let note_m: u8 = note.into();
        let n = (root_m as i8 - note_m as i8) % 12;
        if n > 6 {
            n - 12
        } else {
            n
        }
    }
}
