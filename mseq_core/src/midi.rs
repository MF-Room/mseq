use crate::note::Note;
use serde::{Deserialize, Serialize};

const CLOCK: u8 = 0xf8;
const START: u8 = 0xfa;
const CONTINUE: u8 = 0xfb;
const STOP: u8 = 0xfc;
const NOTE_ON: u8 = 0x90;
const NOTE_OFF: u8 = 0x80;
const CC: u8 = 0xB0;
const PC: u8 = 0xC0;

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

/// Represents a parsed MIDI instruction.
///
/// This enum defines all supported MIDI messages used for input handling.
#[derive(PartialEq)]
pub enum MidiMessage {
    /// Note Off event. This message is sent when a note is released.
    NoteOff {
        /// MIDI channel (1-16).
        channel: u8,
        /// The MIDI note.
        note: MidiNote,
    },
    /// Note On event. This message is sent when a note is pressed.
    NoteOn {
        /// MIDI channel (1-16).
        channel: u8,
        /// The MIDI note.
        note: MidiNote,
    },
    /// A MIDI Control Change (CC) message.
    CC {
        /// MIDI channel (1-16).
        channel: u8,
        /// The controller number (0–127).
        controller: u8,
        /// The controller value (0–127).
        value: u8,
    },
    /// A MIDI Program Change (PC) message.
    PC {
        /// MIDI channel (1-16).
        channel: u8,
        /// The controller value (0–127).
        value: u8,
    },
    /// Timing Clock. Sent 24 times per quarter note when synchronisation is required.
    ///
    /// Intercepted internally for transport synchronization.
    Clock,
    /// Start. Start the current sequence playing.
    ///
    /// Intercepted internally for transport synchronization.
    Start,
    /// Continue. Continue at the point the sequence was Stopped.
    ///
    /// Intercepted internally for transport synchronization.
    Continue,
    /// Stop. Stop the current sequence.
    ///
    /// Intercepted internally for transport synchronization.
    Stop,
}

impl MidiMessage {
    /// Parses a byte slice into a `MidiMessage` struct.
    ///
    /// This function is not intended to be called directly by end users.  
    /// It is used internally to ensure consistent MIDI message parsing logic across platforms.
    ///
    /// Returns `Some(MidiMessage)` if the byte slice represents a known and valid MIDI message,
    /// or `None` if the data does not match any recognized MIDI message format.
    pub fn parse(bytes: &[u8]) -> Option<MidiMessage> {
        if bytes.len() == 1 {
            match bytes[0] {
                CLOCK => Some(MidiMessage::Clock),
                START => Some(MidiMessage::Start),
                CONTINUE => Some(MidiMessage::Continue),
                STOP => Some(MidiMessage::Stop),
                _ => None,
            }
        } else if bytes.len() == 2 && bytes[0] & 0xF0 == PC {
            let channel = (bytes[0] & 0x0F) + 1;
            if is_valid_channel(channel) {
                Some(MidiMessage::PC {
                    channel,
                    value: bytes[1],
                })
            } else {
                None
            }
        } else if bytes.len() == 3 {
            let channel = (bytes[0] & 0x0F) + 1;
            if is_valid_channel(channel) {
                match bytes[0] & 0xF0 {
                    NOTE_OFF => Some(MidiMessage::NoteOff {
                        channel,
                        note: MidiNote::from_midi_value(bytes[1], bytes[2]),
                    }),
                    NOTE_ON => Some(MidiMessage::NoteOn {
                        channel,
                        note: MidiNote::from_midi_value(bytes[1], bytes[2]),
                    }),
                    CC => Some(MidiMessage::CC {
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

/// Performs a linear conversion from `[0.0, 1.0]` to [0, 127]. If `v` is smaller than `0.0` return
/// 0. If `v` is greater than `1.0` return 127. The main purpose of this function is to be used with
/// MIDI control changes (CC).
pub fn param_value(v: f32) -> u8 {
    if v < -1.0 {
        return 0;
    }
    if v > 1.0 {
        return 127;
    }
    63 + (v * 63.0) as u8
}
