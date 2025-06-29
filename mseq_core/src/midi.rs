use crate::note::Note;
use serde::{Deserialize, Serialize};

pub(crate) fn is_valid_channel(channel: u8) -> bool {
    (1..=16).contains(&channel)
}

/// Note that can be sent through a MIDI message.
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Deserialize, Serialize, Hash)]
pub struct MidiNote {
    /// The chromatic note (A to G)
    pub note: Note,
    /// The octave of the note
    pub octave: u8,
    /// The velocity of the note (0 to 127)
    pub vel: u8,
}

impl MidiNote {
    /// Construct a new [`MidiNote`]
    pub fn new(note: Note, octave: u8, vel: u8) -> Self {
        Self { note, octave, vel }
    }

    /// Convert a MIDI note value into a [`MidiNote`].
    pub fn from_midi_value(midi_value: u8, vel: u8) -> Self {
        let octave = midi_value / 12;
        let note = Note::from(midi_value % 12);
        Self::new(note, octave, vel)
    }

    /// Transpose the [`MidiNote`].
    /// The `transpose` parameter corresponds to the number of semitones to add to the note.
    pub fn transpose(&self, transpose: i8) -> Self {
        let (note, octave) = self.note.add_semitone(self.octave, transpose);
        Self {
            note,
            octave,
            vel: self.vel,
        }
    }

    /// Retrieve the MIDI value of the MidiNote, which can be sent through a MIDI message.
    pub fn midi_value(&self) -> u8 {
        u8::from(self.note) + 12 * self.octave
    }
}

/// Midi Message representation according to the Midi Standard
#[derive(PartialEq)]
pub enum MidiMessage {
    /// Note Off event. This message is sent when a note is released.
    NoteOff { channel: u8, note: MidiNote },
    /// Note On event. This message is sent when a note is pressed.
    NoteOn { channel: u8, note: MidiNote },
    /// Control Change. This message is sent when a controller value changes.
    CC {
        channel: u8,
        /// Controller number
        controller: u8,
        /// The new value
        value: u8,
    },
    /// Program Change.
    PC { channel: u8, value: u8 },
    /// Timing Clock. Sent 24 times per quarter note when synchronisation is required.
    Clock,
    /// Start. Start the current sequence playing.
    Start,
    /// Continue. Continue at the point the sequence was Stopped.
    Continue,
    /// Stop. Stop the current sequence.
    Stop,
}

impl MidiMessage {
    pub fn parse(bytes: &[u8]) -> Option<MidiMessage> {
        if bytes.len() == 1 {
            match bytes[0] {
                0xF8 => Some(MidiMessage::Clock),
                0xFA => Some(MidiMessage::Start),
                0xFB => Some(MidiMessage::Continue),
                0xFC => Some(MidiMessage::Stop),
                _ => None,
            }
        } else if bytes.len() == 2 && bytes[0] & 0xF0 == 0xC0 {
            let channel = bytes[0] & 0x0F + 1;
            if is_valid_channel(channel) {
                Some(MidiMessage::PC {
                    channel,
                    value: bytes[1],
                })
            } else {
                None
            }
        } else if bytes.len() == 3 {
            let channel = bytes[0] & 0x0F + 1;
            if is_valid_channel(channel) {
                match bytes[0] & 0xF0 {
                    0x80 => Some(MidiMessage::NoteOff {
                        channel,
                        note: MidiNote::from_midi_value(bytes[1], bytes[2]),
                    }),
                    0x90 => Some(MidiMessage::NoteOn {
                        channel,
                        note: MidiNote::from_midi_value(bytes[1], bytes[2]),
                    }),
                    0xB0 => Some(MidiMessage::CC {
                        channel,
                        controller: bytes[1],
                        value: bytes[2],
                    }),
                    _ => None,
                }
            } else {
                None
            }
        } else {
            None
        }
    }
}
