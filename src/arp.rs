use crate::{DeteTrack, MSeqError, MidiNote, Note};
use std::path::Path;

#[derive(Default, Clone, Copy)]
pub enum ArpDiv {
    #[default]
    T4,
    T8,
    T16,
}

impl DeteTrack {
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
