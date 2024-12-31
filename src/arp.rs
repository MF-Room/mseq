use crate::{DeteTrack, MidiNote, Note};

#[cfg(not(feature = "embedded"))]
use {crate::MSeqError, std::path::Path};

#[cfg(feature = "embedded")]
use crate::embedded_mod::*;

/// Time division of the arpeggiator
#[derive(Default, Clone, Copy)]
pub enum ArpDiv {
    #[default]
    /// Play every quarter note.
    T4,
    /// Play every eighth note.
    T8,
    /// Play every sixteenth note.
    T16,
}

impl DeteTrack {
    /// Create a new arpeggiator track following the notes in `pattern` with the `div` time
    /// division. The `root` note is used for transposition. The track  will be played on the MIDI
    /// channel with `channel_id`.
    pub fn new_arp(
        pattern: Vec<MidiNote>,
        div: ArpDiv,
        root: Note,
        channel_id: u8,
        name: &str,
    ) -> Self {
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

    /// Load an arpeggiator track from a csv file (`filename`) and a time division (`div`). Refer to
    /// this [`example`] for an example file. The `root` note is used for transposition. The track
    /// will be played on the MIDI channel with `channel_id`.
    ///
    /// [`example`]: https://github.com/MF-Room/mseq/tree/main/examples/res/arp_0.csv
    #[cfg(not(feature = "embedded"))]
    pub fn load_arp_from_file<P: AsRef<Path>>(
        filename: P,
        div: ArpDiv,
        root: Note,
        channel_id: u8,
        name: &str,
    ) -> Result<Self, MSeqError> {
        let mut rdr = csv::Reader::from_path(filename)?;
        let pattern = rdr
            .deserialize::<MidiNote>()
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self::new_arp(pattern, div, root, channel_id, name))
    }
}
