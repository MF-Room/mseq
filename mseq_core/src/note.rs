use core::{convert, fmt};

/// Represents 1 note of the chromatic scale.
#[derive(Debug, Default, Clone, PartialEq, Copy, Eq, serde::Deserialize)]
pub enum Note {
    #[default]
    /// C
    C,
    /// C# or Db
    CS,
    /// D
    D,
    /// D# or Eb
    DS,
    /// E
    E,
    /// F
    F,
    /// F# or Gb
    FS,
    /// G
    G,
    /// G# or Ab
    GS,
    /// A
    A,
    /// A# or Bb
    AS,
    /// B
    B,
}

impl convert::From<Note> for u8 {
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

impl fmt::Display for Note {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
    /// Add semitones to a Note and an octave. Returns the new note and octave.
    /// Works with negative semitones.
    ///
    /// # Example
    /// ```
    /// use mseq_core::Note;
    ///
    /// let note = Note::G;
    /// let octave = 4;
    /// let (new_note, new_octave) = note.add_semitone(octave, 5);
    /// assert!(new_note == Note::C);
    /// assert!(new_octave == 5);
    /// ```
    pub fn add_semitone(self, octave: u8, semi: i8) -> (Self, u8) {
        let new_note = u8::from(self) as i8 + semi;
        let q = new_note.div_euclid(12);
        let r = new_note.rem_euclid(12);
        ((r as u8).into(), (octave as i8 + q) as u8)
    }

    /// Number of semitones required to transpose from root to note. The results range from 6 to -5
    /// to minimize pitch difference with the original note.
    pub fn transpose(root: Note, note: Note) -> i8 {
        let root_m: u8 = root.into();
        let note_m: u8 = note.into();
        let n = (note_m as i8 - root_m as i8).rem_euclid(12);
        if n > 6 {
            n - 12
        } else {
            n
        }
    }
}
