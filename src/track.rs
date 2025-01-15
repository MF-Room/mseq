use log::warn;

#[cfg(not(feature = "embedded"))]
use std::path::Path;

#[cfg(feature = "embedded")]
use crate::embedded_mod::*;
#[cfg(not(feature = "embedded"))]
use {crate::MSeqError, log::debug, std::collections::HashMap, thiserror::Error};

use crate::{midi_controller::MidiController, note::Note};
use crate::{MidiNote, MidiOut};

#[derive(Error, Debug)]
pub enum TrackError {
    #[cfg(not(feature = "embedded"))]
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

/// The Track trait can be implemented by the client. A struct with the Track trait can be passed to
/// the MidiController to play it through a midi connection. This allows to reduce the amount of
/// code in the Conductor by writing each track independently.
pub trait Track {
    /// Implement what the track should play at that step. See `examples/impl_track.rs` for an
    /// example usage. Implementation required.
    fn play_step(&mut self, step: u32, midi_controller: &mut MidiController<impl MidiOut>);
    /// Transpose the track. The default implementation returns a warning. Optional implementation.
    fn transpose(&mut self, _note: Option<Note>) {
        warn!("Track::transpose() not implemented")
    }
    /// Returns the root of the track. Optional implementation.
    fn get_root(&self) -> Note {
        Note::C
    }
    /// Set the start step of the track. Optional implementation.
    fn set_start_step(&mut self, _start_step: u32) {
        warn!("Track::set_start_step() not implemented")
    }
    /// Returns the name of the track. Optional implementation.
    fn get_name(&self) -> String {
        "Unamed".to_string()
    }
}

/// DeteTrack implements the Track trait, so it can be passed to the MidiController to play it. It
/// is defined by a list of notes that will always play at the same time in the track, hence the
/// name (Deterministic Track).
#[derive(Default, Clone)]
pub struct DeteTrack {
    len: u32,
    notes: Vec<(MidiNote, u32, u32)>, // (Note, start step, length)
    start_step: u32,
    root: Note,
    transpose: Option<i8>,
    channel_id: u8,
    name: String,
}

impl Track for DeteTrack {
    fn play_step(&mut self, step: u32, midi_controller: &mut MidiController<impl MidiOut>) {
        let cur_step = step % self.len;
        for n in &self.notes {
            if (n.1 + self.start_step) % self.len == cur_step {
                let note = self.transpose.map_or(n.0, |t| n.0.transpose(t));
                midi_controller.play_note(note, n.2, self.channel_id)
            }
        }
    }

    fn transpose(&mut self, note: Option<Note>) {
        self.transpose = note.map(|n| Note::transpose(self.root, n));
    }

    fn get_root(&self) -> Note {
        self.root
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn set_start_step(&mut self, start_step: u32) {
        self.start_step = start_step;
    }
}

impl DeteTrack {
    /// Create a new DeteTrack from a list of notes, its length, the midi channel and a name.
    /// Specify the root note to allow transposition.
    pub fn new(
        len: u32,
        notes: Vec<(MidiNote, u32, u32)>,
        root: Note,
        channel_id: u8,
        name: &str,
    ) -> Self {
        DeteTrack {
            len,
            notes,
            start_step: 0,
            root,
            transpose: None,
            channel_id,
            name: name.to_string(),
        }
    }

    /// Set the root of the DeteTrack. This function does not transpose the track, it only changes
    /// the root note.
    pub fn set_root(&mut self, note: Note) {
        self.root = note;
    }

    /// Load an acid track from a midi file. Refer to `examples/midi_track.rs` for an example usage.
    /// Provide the root note of the track to allow for transposition. channel_id is the midi
    /// channel where this track will be played when passed to the MidiController.
    #[cfg(not(feature = "embedded"))]
    pub fn load_from_file<P: AsRef<Path>>(
        filename: P,
        root: Note,
        channel_id: u8,
        name: &str,
    ) -> Result<Self, MSeqError> {
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

    /// Return the all `(note, length)`, that start at `step`. Transposition and start step are
    /// taken into account.
    pub fn get_notes_start_at_step(&self, step: u32) -> Vec<(MidiNote, u32)> {
        let mut notes = vec![];
        let cur_step = step % self.len;
        for n in &self.notes {
            if (n.1 + self.start_step) % self.len == cur_step {
                let note = self.transpose.map_or(n.0, |t| n.0.transpose(t));
                notes.push((note, n.2));
            }
        }
        notes
    }
}
