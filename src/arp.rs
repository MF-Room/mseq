use crate::{DeteTrack, MSeqError, MidiNote, Note};
use std::path::Path;

/// Time division of the arpeggiator
#[derive(Default, Clone, Copy)]
pub enum ArpDiv {
    #[default]
    /// Play every whole noted
    T4,
    /// Play every half note
    T8,
    /// Play every quarter note
    T16,
}

impl DeteTrack {
    /// Create a new arpeggiator track from a list of notes, a time division, the root note, the
    /// midi channel and a name.
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

    /// Load an arpeggiator track from a csv file and a time division. Refer to
    /// `examples/res/arp_0.csv` for an example file. Provide the root note of the track to allow
    /// for transposition. channel_id is the midi channel where this track will be played when
    /// passed to the MidiController.
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
