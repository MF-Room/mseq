use log::{debug, warn};
use std::{collections::HashMap, path::Path};
use thiserror::Error;

use mseq_core::{DeteTrack, MidiNote, Note};

use crate::MSeqError;

#[derive(Error, Debug)]
pub enum TrackError {
    #[error("Failed to read midi file: {0}")]
    Io(#[from] std::io::Error),
    #[error("Midly error: {0}")]
    Midly(#[from] midly::Error),
    #[error("Cannot play the same note before it ends")]
    DuplicateNote,
    #[error("Cannot end a note before playing it")]
    WrongNoteOff,
    #[error("Midi file doesn't contain a single track")]
    BadFormat,
    #[error("Unsupported timing specification")]
    BadTiming,
}

/// Load an acid track from a midi file. Refer to `examples/midi_track.rs` for an example usage.
/// Provide the root note of the track to allow for transposition. channel_id is the midi
/// channel where this track will be played when passed to the MidiController.
pub fn load_from_file<P: AsRef<Path>>(
    filename: P,
    root: Note,
    channel_id: u8,
    name: &str,
) -> Result<DeteTrack, MSeqError> {
    let bytes = fs_err::read(filename).map_err(|e| MSeqError::Track(TrackError::Io(e)))?;
    let smf = midly::Smf::parse(&bytes).map_err(|e| MSeqError::Track(TrackError::Midly(e)))?;

    match smf.header.format {
        midly::Format::SingleTrack => (),
        _ => return Err(MSeqError::Track(TrackError::BadFormat)),
    }

    let mut notes_map: HashMap<u8, (u8, u32, u32)> = HashMap::new();
    let mut notes: Vec<(MidiNote, u32, u32)> = vec![];
    let mut step = 0;

    // 24 comes from the TimeSignature (number of clocks per beat)
    let step_size = u16::from(if let midly::Timing::Metrical(s) = smf.header.timing {
        s
    } else {
        return Err(MSeqError::Track(TrackError::BadTiming));
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
                        .remove_entry(&key.into())
                        .ok_or(TrackError::WrongNoteOff)?;
                    notes.push((MidiNote::from_midi_value(midi_value, vel), start, duration));
                }
                midly::MidiMessage::NoteOn { key, vel } => {
                    if notes_map
                        .insert(key.into(), (vel.into(), step, 0))
                        .is_some()
                    {
                        return Err(MSeqError::Track(TrackError::DuplicateNote));
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
