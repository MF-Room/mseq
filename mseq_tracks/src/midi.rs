use alloc::vec;
use alloc::vec::Vec;
use hashbrown::HashMap;

use log::{debug, warn};

use mseq_core::{DeteTrack, MidiNote, Note};

use crate::TrackError;

#[cfg(feature = "std")]
use std::path::Path;

/// Load an acid track from a midi file. Refer to `examples/midi_track.rs` for an example usage.
/// Provide the root note of the track to allow for transposition. channel_id is the midi
/// channel where this track will be played when passed to the MidiController.
#[cfg(feature = "std")]
pub fn load_from_file<P: AsRef<Path>>(
    filename: P,
    root: Note,
    channel_id: u8,
    name: &str,
) -> Result<DeteTrack, TrackError> {
    let bytes = fs_err::read(filename).map_err(TrackError::Io)?;
    let smf = midly::Smf::parse(&bytes).map_err(TrackError::Midly)?;

    match smf.header.format {
        midly::Format::SingleTrack => (),
        _ => return Err(TrackError::BadFormat),
    }

    let mut notes_map: HashMap<u8, (u8, u32, u32)> = HashMap::new();
    let mut notes: Vec<(MidiNote, u32, u32)> = vec![];
    let mut step = 0;

    // 24 comes from the TimeSignature (number of clocks per beat)
    let step_size = u16::from(if let midly::Timing::Metrical(s) = smf.header.timing {
        s
    } else {
        return Err(TrackError::BadTiming);
    }) as u32
        / 24;

    debug!("{:?}", smf.header.timing);
    let track = smf.tracks.first().ok_or(TrackError::BadFormat)?;

    for event in track {
        debug!("step: {}, event: {:?}", step, event);
        let nb_clocks = u32::from(event.delta) / step_size;

        // Increase duration of all the current notes
        notes_map
            .values_mut()
            .for_each(|(_vel, _start, dur)| *dur += nb_clocks);
        step += nb_clocks;

        match event.kind {
            midly::TrackEventKind::Midi {
                channel: _,
                message,
            } => match message {
                midly::MidiMessage::NoteOff { key, vel: _ } => {
                    let (midi_value, (vel, start, duration)) = notes_map
                        .remove_entry::<u8>(&key.into())
                        .ok_or(TrackError::WrongNoteOff)?;
                    notes.push((MidiNote::from_midi_value(midi_value, vel), start, duration));
                }
                midly::MidiMessage::NoteOn { key, vel } => {
                    if notes_map
                        .insert(key.into(), (vel.into(), step, 0))
                        .is_some()
                    {
                        return Err(TrackError::DuplicateNote);
                    }
                }
                _ => warn!("Unsupported midi event: {:?}", event),
            },
            midly::TrackEventKind::Meta(m) => {
                if m == midly::MetaMessage::EndOfTrack {
                    break;
                } else {
                    warn!("Unsupported midi event: {:?}", event);
                }
            }
            _ => warn!("Unsupported midi event: {:?}", event),
        }
    }
    Ok(DeteTrack::new(step, notes, root, channel_id, name))
}
