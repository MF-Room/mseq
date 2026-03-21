use alloc::vec;
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::io::Read;

use mseq_core::{DeteTrack, MidiNote, Note};

#[derive(Default, Clone, Copy, Debug, serde::Deserialize)]
/// Timing mostly used in [`AcidTrig`] to generate acid tracks.
pub enum Timing {
    /// Play a note.
    Note,
    #[default]
    /// Rest.
    Rest,
    /// Tie to the previous note, if the previous note is a [`Rest`] then the note is not played.
    Tie,
}

/// Trig used to create acid tracks with [`new`]. Each Trig represents one sixteenth
/// step. `AcidTrig` provides a similar interface to the original [`Roland TB-303`] with some slight
/// modifications.
///
/// [`Roland TB-303`]: https://en.wikipedia.org/wiki/Roland_TB-303
#[derive(Debug, serde::Deserialize)]
pub struct AcidTrig {
    #[serde(flatten)]
    /// Pitch and Velocity
    pub midi_note: MidiNote,
    /// Timing
    pub timing: Timing,
}

use Timing::*;

/// Create a new acid track following the trigs in `pattern`. The `root` note is used for
/// transposition. The track  will be played on the MIDI channel with `channel_id`.
pub fn new(pattern: Vec<AcidTrig>, root: Note, channel_id: u8, name: &str) -> DeteTrack {
    if pattern.is_empty() {
        return DeteTrack::new(0, vec![], root, channel_id, name);
    }
    //(note, start, tie_counter)
    let mut prev_note: Option<(MidiNote, u32, u32)> = None;
    let mut notes = vec![];
    for (step, trig) in pattern.iter().enumerate() {
        let step = step as u32;
        match trig.timing {
            Note => {
                if let Some(n) = prev_note {
                    notes.push((n.0, 6 * n.1, 3 + 6 * n.2));
                }
                prev_note = Some((trig.midi_note, step, 0));
            }
            Rest => {
                if let Some(n) = prev_note {
                    notes.push((n.0, 6 * n.1, 3 + 6 * n.2));
                }
                prev_note = None;
            }
            Tie => {
                prev_note = if let Some(n) = prev_note {
                    if n.0.note == trig.midi_note.note && n.0.octave == trig.midi_note.octave {
                        Some((n.0, n.1, n.2 + 1))
                    } else {
                        notes.push((n.0, 6 * n.1, 7 + 6 * n.2));
                        Some((trig.midi_note, step, 0))
                    }
                } else {
                    None
                };
            }
        }
    }

    if let Some(n) = prev_note {
        notes.push((n.0, 6 * n.1, 3 + 6 * n.2));
    }

    DeteTrack::new(6 * pattern.len() as u32, notes, root, channel_id, name)
}

#[cfg(feature = "std")]
use std::path::Path;

#[cfg(feature = "std")]
use crate::TrackError;

/// Load an acid track from a csv file (`filename`). Refer to this [`example`] for an example
/// file. The `root` note is used for transposition. The track will be played on the MIDI
/// channel with `channel_id`.
///
/// [`example`]: https://github.com/MF-Room/mseq/tree/main/examples/res/acid_0.csv
#[cfg(feature = "std")]
pub fn load_from_file<P: AsRef<Path>>(
    filename: P,
    root: Note,
    channel_id: u8,
    name: &str,
) -> Result<DeteTrack, TrackError> {
    let mut rdr = csv::Reader::from_path(filename)?;
    load_from_reader(&mut rdr, root, channel_id, name)
}
#[cfg(feature = "std")]
pub(crate) fn load_from_reader<R: Read>(
    rdr: &mut csv::Reader<R>,
    root: Note,
    channel_id: u8,
    name: &str,
) -> Result<DeteTrack, TrackError> {
    let pattern = rdr
        .deserialize::<AcidTrig>()
        .collect::<Result<Vec<_>, _>>()?;
    Ok(new(pattern, root, channel_id, name))
}
