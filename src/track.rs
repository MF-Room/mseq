use log::{debug, warn};
use std::collections::HashMap;
use std::path::Path;

use thiserror::Error;

use crate::{midi_controller::MidiController, note::Note};
use crate::{MidiConnection, MidiNote};

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

pub trait Track {
    fn play_step(&mut self, step: u32, midi_controller: &mut MidiController<impl MidiConnection>);
    fn transpose(&mut self, _note: Option<Note>) {
        warn!("Track::transpose() not implemented")
    }
    fn get_root(&self) -> Note {
        Note::C
    }
    fn set_start_step(&mut self, _start_step: u32) {
        warn!("Track::set_start_step() not implemented")
    }

    fn get_name(&self) -> String {
        "Unamed".to_string()
    }
}

#[derive(Default, Clone)]
pub struct DeteTrack {
    len: u32,
    notes: Vec<(MidiNote, u32, u32)>, // (Note, start step, length)
    pub start_step: u32,
    root: Note,
    transpose: Option<i8>,
    pub channel_id: u8,
    name: String,
}

impl Track for DeteTrack {
    fn play_step(&mut self, step: u32, midi_controller: &mut MidiController<impl MidiConnection>) {
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
}

impl DeteTrack {
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

    pub fn set_root(&mut self, note: Note) {
        self.root = note;
    }

    pub fn load_from_file<P: AsRef<Path>>(
        filename: P,
        root: Note,
        channel_id: u8,
        name: &str,
    ) -> Result<Self, TrackError> {
        let bytes = fs_err::read(filename)?;
        let smf = midly::Smf::parse(&bytes)?;

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
                            .remove_entry(&key.into())
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
}
