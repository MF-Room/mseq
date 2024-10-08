use crate::{DeteTrack, MSeqError, MidiNote, Note};
use std::path::Path;

#[derive(Default, Clone, Copy, Debug, serde::Deserialize)]

/// Timing mostly used in [`AcidTrig`] to generate acid tracks.
pub enum Timing {
    /// Play a note.
    Note,
    #[default]
    /// Rest.
    Rest,
}

/// Trig used to create acid tracks with [`DeteTrack::new_acid`]. Each Trig represents one
/// sixteenth step. `AcidTrig` provides a similar interface to the original [`Roland TB-303`] with
/// some slight modification.
///
/// [`Roland TB-303`]: https://en.wikipedia.org/wiki/Roland_TB-303
#[derive(Debug, serde::Deserialize)]
pub struct AcidTrig {
    #[serde(flatten)]
    /// Pitch and Velocity
    pub midi_note: MidiNote,
    /// Slide enable
    pub slide: bool,
    /// Timing
    pub timing: Timing,
}

use Timing::*;

impl DeteTrack {
    /// Create a new acid track from a pattern, its root note, the midi channel, and a name.
    pub fn new_acid(pattern: Vec<AcidTrig>, root: Note, channel_id: u8, name: &str) -> Self {
        if pattern.is_empty() {
            return DeteTrack::new(0, vec![], root, channel_id, name);
        }
        //(note, start, glide, tie_counter)
        let mut prev_note: Option<(MidiNote, u32, bool, u32)> = None;
        let mut notes = vec![];
        for (step, trig) in pattern.iter().enumerate() {
            let step = step as u32;
            match trig.timing {
                Note => {
                    prev_note = Some(if let Some(n) = prev_note {
                        if n.2 {
                            if n.0.note == trig.midi_note.note
                                && n.0.octave == trig.midi_note.octave
                            {
                                (n.0, n.1, trig.slide, n.3 + 1)
                            } else {
                                notes.push((n.0, n.1, 7 + 6 * n.3));
                                (trig.midi_note, 6 * step, trig.slide, 0)
                            }
                        } else {
                            notes.push((n.0, n.1, 3 + 6 * n.3));
                            (trig.midi_note, 6 * step, trig.slide, 0)
                        }
                    } else {
                        (trig.midi_note, 6 * step, trig.slide, 0)
                    });
                }
                Rest => {
                    if let Some(n) = prev_note {
                        notes.push((n.0, n.1, 3 + 6 * n.3));
                    }
                    prev_note = None;
                }
            }
        }

        match pattern[0].timing {
            Note => {
                if let Some(n) = prev_note {
                    if n.2 {
                        if n.0.note == pattern[0].midi_note.note
                            && n.0.octave == pattern[0].midi_note.octave
                        {
                            notes.push((n.0, n.1, 3 + 6 * n.3));
                        } else {
                            notes.push((n.0, n.1, 7 + 6 * n.3));
                        }
                    } else {
                        notes.push((n.0, n.1, 3 + 6 * n.3));
                    }
                }
            }
            Rest => {
                if let Some(n) = prev_note {
                    notes.push((n.0, n.1, 3 + 6 * n.3));
                }
            }
        };

        DeteTrack::new(6 * pattern.len() as u32, notes, root, channel_id, name)
    }

    /// Load an acid track from a csv file. Refer to `examples/res/acid_0.csv` for an example file.
    /// Provide the root note of the track to allow for transposition. channel_id is the midi
    /// channel where this track will be played when passed to the MidiController.
    pub fn load_acid_from_file<P: AsRef<Path>>(
        filename: P,
        root: Note,
        channel_id: u8,
        name: &str,
    ) -> Result<Self, MSeqError> {
        let mut rdr = csv::Reader::from_path(filename)?;
        let pattern = rdr
            .deserialize::<AcidTrig>()
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self::new_acid(pattern, root, channel_id, name))
    }
}
