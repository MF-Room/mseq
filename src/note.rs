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
    pub fn transpose_from_c(&self) -> i8 {
        match self {
            Note::C => 0,
            Note::CS => 1,
            Note::D => 2,
            Note::DS => 3,
            Note::E => 4,
            Note::F => 5,
            Note::FS => -6,
            Note::G => -5,
            Note::GS => -4,
            Note::A => -3,
            Note::AS => -2,
            Note::B => -1,
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
