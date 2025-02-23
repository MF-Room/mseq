use crate::{DeteTrack, MidiNote, Note};

#[cfg(feature = "std")]
use {crate::MSeqError, std::path::Path};

#[cfg(not(feature = "std"))]
use crate::no_std_mod::*;

#[derive(Default, Clone, Copy, Debug)]
#[cfg_attr(feature = "std", derive(serde::Deserialize))]
/// Timing mostly used in [`AcidTrig`] to generate acid tracks.
pub enum Timing {
    /// Play a note.
    Note,
    #[default]
    /// Rest.
    Rest,
}

/// Trig used to create acid tracks with [`DeteTrack::new_acid`]. Each Trig represents one sixteenth
/// step. `AcidTrig` provides a similar interface to the original [`Roland TB-303`] with some slight
/// modifications.
///
/// [`Roland TB-303`]: https://en.wikipedia.org/wiki/Roland_TB-303
#[derive(Debug)]
#[cfg_attr(feature = "std", derive(serde::Deserialize))]
pub struct AcidTrig {
    #[cfg_attr(feature = "std", serde(flatten))]
    /// Pitch and Velocity
    pub midi_note: MidiNote,
    /// Slide enable
    pub slide: bool,
    /// Timing
    pub timing: Timing,
}

use Timing::*;

impl DeteTrack {
    /// Create a new acid track following the trigs in `pattern`. The `root` note is used for
    /// transposition. The track  will be played on the MIDI channel with `channel_id`.
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

    /// Load an acid track from a csv file (`filename`). Refer to this [`example`] for an example
    /// file. The `root` note is used for transposition. The track will be played on the MIDI
    /// channel with `channel_id`.
    ///
    /// [`example`]: https://github.com/MF-Room/mseq/tree/main/examples/res/acid_0.csv
    #[cfg(feature = "std")]
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
