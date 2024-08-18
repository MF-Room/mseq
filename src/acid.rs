use crate::DeteTrack;
use crate::MidiNote;
use crate::Note;

#[derive(Default, Clone, Copy)]
pub enum Timing {
    Note,
    Tie,
    #[default]
    Rest,
}

use Timing::*;

impl DeteTrack {
    /// pattern: Vec<((Note, octave), velocity, slide, timing)>
    pub fn new_acid(
        pattern: Vec<((Note, u8), u8, bool, Timing)>,
        root: Note,
        channel_id: u8,
        name: &str,
    ) -> Self {
        let mut prev_note: Option<(MidiNote, u32, bool)> = None;
        let mut step = 0;
        let mut notes = vec![];
        for trig in &pattern {
            match trig.3 {
                Note => {
                    prev_note.map(|n| {
                        let length = if n.2 { 7 } else { 3 };
                        notes.push((n.0, n.1, length));
                    });
                    prev_note = Some((MidiNote::new(trig.0 .0, trig.0 .1, trig.1), step, trig.2));
                }
                Tie => {
                    prev_note.map(|n| {
                        notes.push((n.0, n.1, 7));
                    });
                    prev_note = prev_note
                        .map(|n| (MidiNote::new(n.0.note, n.0.octave, trig.1), step, trig.2));
                }
                Rest => {
                    prev_note.map(|n| {
                        notes.push((n.0, n.1, 3));
                    });
                    prev_note = None;
                }
            }
            step += 1;
        }

        match pattern[0].3 {
            Note => {
                prev_note.map(|n| {
                    let length = if n.2 { 7 } else { 3 };
                    notes.push((n.0, n.1, length));
                });
            }
            Tie => {
                prev_note.map(|n| {
                    notes.push((n.0, n.1, 3));
                });
            }
            Rest => {
                prev_note.map(|n| {
                    notes.push((n.0, n.1, 3));
                });
            }
        };

        DeteTrack::new(pattern.len() as u32, notes, root, channel_id, name)
    }
}
