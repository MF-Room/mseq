use alloc::vec::Vec;
use serde::Deserialize;
#[cfg(feature = "std")]
use std::io::Read;

use mseq_core::{DeteTrack, MidiNote, Note};

/// Time division of the arpeggiator
#[derive(Default, Clone, Copy, Debug, Deserialize)]
pub enum ArpDiv {
    #[default]
    /// Play every quarter note.
    T4,
    /// Play every eighth note.
    T8,
    /// Play every sixteenth note.
    T16,
}

/// Create a new arpeggiator track following the notes in `pattern` with the `div` time
/// division. The `root` note is used for transposition. The track  will be played on the MIDI
/// channel with `channel_id`.
pub fn new(
    pattern: Vec<MidiNote>,
    div: ArpDiv,
    root: Note,
    channel_id: u8,
    name: &str,
) -> DeteTrack {
    let factor = match div {
        ArpDiv::T4 => 24,
        ArpDiv::T8 => 12,
        ArpDiv::T16 => 6,
    };

    let notes = pattern
        .iter()
        .enumerate()
        .map(|(s, t)| (*t, factor * s as u32, factor / 2))
        .collect();
    let len = pattern.len() as u32 * factor;
    DeteTrack::new(len, notes, root, channel_id, name)
}

#[cfg(feature = "std")]
use std::path::Path;

#[cfg(feature = "std")]
use crate::TrackError;
/// Load an arpeggiator track from a csv file (`filename`) and a time division (`div`). Refer to
/// this [`example`] for an example file. The `root` note is used for transposition. The track
/// will be played on the MIDI channel with `channel_id`.
///
/// [`example`]: https://github.com/MF-Room/mseq/tree/main/examples/res/arp_0.csv
#[cfg(feature = "std")]
pub fn load_from_file<P: AsRef<Path>>(
    filename: P,
    div: ArpDiv,
    root: Note,
    channel_id: u8,
    name: &str,
) -> Result<DeteTrack, TrackError> {
    let mut rdr = csv::Reader::from_path(filename)?;
    load_from_reader(&mut rdr, div, root, channel_id, name)
}

#[cfg(feature = "std")]
pub(crate) fn load_from_reader<R: Read>(
    rdr: &mut csv::Reader<R>,
    div: ArpDiv,
    root: Note,
    channel_id: u8,
    name: &str,
) -> Result<DeteTrack, TrackError> {
    let pattern = rdr
        .deserialize::<MidiNote>()
        .collect::<Result<Vec<_>, _>>()?;
    Ok(new(pattern, div, root, channel_id, name))
}
