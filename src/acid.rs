use crate::DeteTrack;
use crate::MidiNote;
use crate::Note;

#[derive(Default, Clone, Copy)]
pub enum Timing {
    Note,
    #[default]
    Rest,
}

use Timing::*;

impl DeteTrack {
    pub fn new_acid(
        pattern: Vec<(MidiNote, bool, Timing)>,
        root: Note,
        channel_id: u8,
        name: &str,
    ) -> Self {
        let mut prev_note: Option<(MidiNote, u32, bool)> = None;
        let mut notes = vec![];
        for (step, trig) in pattern.iter().enumerate() {
            let step = step as u32;
            match trig.2 {
                Note => {
                    if let Some(n) = prev_note {
                        let length = if n.2 { 7 } else { 3 };
                        notes.push((n.0, n.1, length));
                    };
                    prev_note = Some((trig.0, step, trig.1));
                }
                Rest => {
                    if let Some(n) = prev_note {
                        notes.push((n.0, n.1, 3));
                    }
                    prev_note = None;
                }
            }
        }

        match pattern[0].2 {
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
}
