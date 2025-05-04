use alloc::vec;
use alloc::vec::Vec;

use mseq_core::{DeteTrack, MidiNote};

use crate::TrackError;

#[derive(Debug, serde::Deserialize)]
/// Struct used in [`DeteTrack::new_clock_div`] to generate a track with a pattern based on
/// clock divisions.
pub struct ClockDiv {
    /// Note triggered every `div` clock messages
    pub div: u32,
    /// Number of clock messages
    pub duration: u32,
}

/// Create a new [`DeteTrack`] with a patern made up of different [`ClockDiv`]. This pattern
/// triggers `note` on the MIDI channel with `channel_id`.
pub fn new(pattern: Vec<ClockDiv>, note: MidiNote, channel_id: u8, name: &str) -> DeteTrack {
    let mut notes = vec![];
    let mut len = 0;
    for p in pattern {
        let nb_trigs = p.duration / p.div;
        for i in 0..nb_trigs {
            notes.push((note, len + i * p.div, p.div));
        }
        len += p.duration;
    }
    DeteTrack::new(len, notes, note.note, channel_id, name)
}

#[cfg(feature = "std")]
use std::path::Path;

/// Load a clock division track from a csv file (`filename`). This pattern
/// triggers `note` on the MIDI channel with `channel_id`. Refer to this [`example`] for an
/// example file.
///
/// [`example`]: https://github.com/MF-Room/mseq/tree/main/examples/res/clk_div_0.csv
#[cfg(feature = "std")]
pub fn load_from_file<P: AsRef<Path>>(
    filename: P,
    note: MidiNote,
    channel_id: u8,
    name: &str,
) -> Result<DeteTrack, TrackError> {
    let mut rdr = csv::Reader::from_path(filename)?;
    let pattern = rdr
        .deserialize::<ClockDiv>()
        .collect::<Result<Vec<_>, _>>()?;
    Ok(new(pattern, note, channel_id, name))
}
