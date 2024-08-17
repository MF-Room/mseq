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
    pub fn get_midi(&self) -> u8 {
        match self {
            Note::C => 60,
            Note::CS => 61,
            Note::D => 62,
            Note::DS => 63,
            Note::E => 64,
            Note::F => 65,
            Note::FS => 54,
            Note::G => 55,
            Note::GS => 56,
            Note::A => 57,
            Note::AS => 58,
            Note::B => 59,
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
