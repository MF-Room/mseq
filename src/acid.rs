use crate::DeteTrack;
use crate::MidiNote;
use crate::Note;
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AcidError {
    #[error("Failed to read acid file [{}: {}]\n\t{0}", file!(), line!())]
    Reading(#[from] csv::Error),
}

#[derive(Default, Clone, Copy, Debug, serde::Deserialize)]
pub enum Timing {
    Note,
    #[default]
    Rest,
}

#[derive(Debug, serde::Deserialize)]
pub struct AcidTrig {
    #[serde(flatten)]
    midi_note: MidiNote,
    slide: bool,
    timing: Timing,
}

use Timing::*;

impl DeteTrack {
    pub fn new_acid(pattern: Vec<AcidTrig>, root: Note, channel_id: u8, name: &str) -> Self {
        if pattern.len() == 0 {
            return DeteTrack::new(0, vec![], root, channel_id, name);
        }
        let mut prev_note: Option<(MidiNote, u32, bool)> = None;
        let mut notes = vec![];
        for (step, trig) in pattern.iter().enumerate() {
            let step = step as u32;
            match trig.timing {
                Note => {
                    if let Some(n) = prev_note {
                        let length = if n.2 { 7 } else { 3 };
                        notes.push((n.0, n.1, length));
                    };
                    prev_note = Some((trig.midi_note, step, trig.slide));
                }
                Rest => {
                    if let Some(n) = prev_note {
                        notes.push((n.0, n.1, 3));
                    }
                    prev_note = None;
                }
            }
        }

        match pattern[0].timing {
            Note => {
                if let Some(n) = prev_note {
                    let length = if n.2 { 7 } else { 3 };
                    notes.push((n.0, n.1, length));
                }
            }
            Rest => {
                if let Some(n) = prev_note {
                    notes.push((n.0, n.1, 3));
                }
            }
        };

        DeteTrack::new(pattern.len() as u32, notes, root, channel_id, name)
    }

    pub fn load_acid_from_file<P: AsRef<Path>>(
        filename: P,
        root: Note,
        channel_id: u8,
        name: &str,
    ) -> Result<Self, AcidError> {
        let mut rdr = csv::Reader::from_path(filename)?;
        let pattern = rdr
            .deserialize::<AcidTrig>()
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self::new_acid(pattern, root, channel_id, name))
    }
}
