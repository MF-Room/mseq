use alloc::vec;
use alloc::vec::Vec;
use core::hash::{Hash, Hasher};

use hashbrown::{HashMap, HashSet};
use log::error;

use crate::MidiMessage;
use crate::midi::{MidiNote, is_valid_channel};
use crate::midi_out::MidiOut;

const MAX_MIDI_CHANNEL: u8 = 16;

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
struct NotePlay {
    midi_note: MidiNote,
    channel_id: u8,
}

impl Hash for NotePlay {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (self.midi_note.midi_value() as u32 + MAX_MIDI_CHANNEL as u32 * self.channel_id as u32)
            .hash(state);
    }
}

/// Represents instructions that can be interpreted and processed by the [`MidiController`].
///
/// These instructions are used to generate and send MIDI messages.
#[derive(PartialEq)]
pub enum Instruction {
    /// Plays a note for a specified duration on a given MIDI channel.
    PlayNote {
        /// The MIDI note to play.
        midi_note: MidiNote,
        /// Duration of the note in MIDI clock ticks (1 tick = 1/24 of a quarter note at the current BPM).
        len: u32,
        /// The MIDI channel to send the message on (1–16).
        channel_id: u8,
    },
    /// Starts playing a MIDI note without specifying a duration.
    ///
    /// Requires a corresponding [`Instruction::StopNote`] to stop the note.
    StartNote {
        /// The MIDI note to start.
        midi_note: MidiNote,
        /// MIDI channel (1–16) to use.
        channel_id: u8,
    },
    /// Stops a previously started MIDI note.
    StopNote {
        /// The MIDI note to stop.
        midi_note: MidiNote,
        /// MIDI channel (1–16) used when the note was started.
        channel_id: u8,
    },
    /// Sends a MIDI Control Change (CC) message.
    SendCC {
        /// MIDI channel (1–16).
        channel_id: u8,
        /// The controller number (0–127).
        parameter: u8,
        /// The controller value (0–127).
        value: u8,
    },
    /// Stops all currently playing notes.
    StopAllNotes,
    /// Sends a raw MIDI message directly.
    MidiMessage {
        /// The raw MIDI message to send.
        midi_message: MidiMessage,
    },
    /// Sends a MIDI Continue message to resume playback.
    ///
    /// This instruction is not intended to be called manually.
    /// Prefer calling [`crate::Context::resume`] instead.
    Continue,
    /// Sends a MIDI Start message to begin playback from the start.
    ///
    /// This instruction is not intended to be called manually.
    /// Prefer calling [`crate::Context::start`] instead.
    Start,
    /// Sends a MIDI Stop message to halt playback.
    ///
    /// This instruction is not intended to be called manually.
    /// Prefer calling [`crate::Context::pause`] instead.
    Stop,
}

impl Instruction {
    /// Transposes the note in the instruction by the given number of `semitones`.
    ///
    /// This function is only applicable when the [`Instruction`] variant is
    /// [`Instruction::PlayNote`], [`Instruction::StartNote`], or [`Instruction::StopNote`].
    pub fn transpose(&mut self, semitones: i8) {
        match self {
            Instruction::PlayNote {
                midi_note,
                len: _,
                channel_id: _,
            } => *midi_note = midi_note.transpose(semitones),
            Instruction::StartNote {
                midi_note,
                channel_id: _,
            } => *midi_note = midi_note.transpose(semitones),
            Instruction::StopNote {
                midi_note,
                channel_id: _,
            } => *midi_note = midi_note.transpose(semitones),
            _ => (),
        }
    }
}

/// Provides a reusable MIDI interface and core logic across different platforms.
///
/// This struct is intended for internal use only and should not be accessed directly by users.
/// Instead, users should:
/// - Use your implementation of [`Conductor`] to send MIDI messages.
/// - Use the [`Context`] to access system settings.
///
/// [`Conductor`]: crate::conductor::Conductor
/// [`Context`]: crate::Context
pub struct MidiController<T: MidiOut> {
    step: u32,

    // Every note currently being played triggered by play_note. The key is the step at which to
    // stop the note.
    play_note_set: HashMap<u32, Vec<NotePlay>>,

    // Every note currently being played triggered by start_note.
    start_note_set: HashSet<NotePlay>,

    // Notes to play at the next update call
    notes_to_play: Vec<NotePlay>,

    midi_out: T,
}

impl<T: MidiOut> MidiController<T> {
    /// This function is not intended to be called directly by the user.
    ///
    /// It exists to facilitate code reuse across different environments and platforms.
    pub fn new(midi_out: T) -> Self {
        Self {
            step: 0,
            play_note_set: HashMap::new(),
            start_note_set: HashSet::new(),
            notes_to_play: vec![],
            midi_out,
        }
    }

    pub(crate) fn execute(&mut self, instruction: Instruction) {
        match instruction {
            Instruction::PlayNote {
                midi_note,
                len,
                channel_id,
            } => self.play_note(midi_note, len, channel_id),
            Instruction::StartNote {
                midi_note,
                channel_id,
            } => self.start_note(midi_note, channel_id),
            Instruction::StopNote {
                midi_note,
                channel_id,
            } => self.stop_note(midi_note, channel_id),
            Instruction::SendCC {
                channel_id,
                parameter,
                value,
            } => self.send_cc(channel_id, parameter, value),
            Instruction::StopAllNotes => self.stop_all_notes(),
            Instruction::Continue => self.send_continue(),
            Instruction::Start => self.start(),
            Instruction::Stop => self.stop(),
            Instruction::MidiMessage { midi_message } => self.send_message(midi_message),
        }
    }

    /// Requests the MIDI controller to play a note at the current MIDI step. Specify the length
    /// (`len`) of the note and the MIDI channel id (`channel_id`) on which to send the note.
    fn play_note(&mut self, midi_note: MidiNote, len: u32, channel_id: u8) {
        if len == 0 || !is_valid_channel(channel_id) {
            return;
        }

        let note_play = NotePlay {
            midi_note,
            channel_id,
        };
        self.notes_to_play.push(note_play);
        self.stop_note_at_step(note_play, self.step + len);
    }

    /// Requests the MIDI controller to start playing a note. Specify the MIDI channel id
    /// (`channel_id`). The note will not stop until [`MidiController::stop_note`] is called with
    /// the same note, ocatve and MIDI channel id.
    fn start_note(&mut self, midi_note: MidiNote, channel_id: u8) {
        if !is_valid_channel(channel_id) {
            return;
        }
        let note_play = NotePlay {
            midi_note,
            channel_id,
        };
        self.notes_to_play.push(note_play);
        self.start_note_set.insert(note_play);
    }

    /// Requests the MIDI controller to stop playing a note that was started by
    /// [`MidiController::start_note`]. The note will stop only if the note, octave and MIDI
    /// channel are identical to what was used in [`MidiController::start_note`].
    fn stop_note(&mut self, midi_note: MidiNote, channel_id: u8) {
        if !is_valid_channel(channel_id) {
            return;
        }
        let note_play = NotePlay {
            midi_note,
            channel_id,
        };
        self.start_note_set.remove(&note_play);
        self.stop_note_at_step(note_play, self.step);
    }

    fn stop_note_at_step(&mut self, note_play: NotePlay, step: u32) {
        self.play_note_set.entry(step).or_default().push(note_play);
    }

    fn send_cc(&mut self, channel_id: u8, parameter: u8, value: u8) {
        if !is_valid_channel(channel_id) {
            return;
        }
        if let Err(e) = self.midi_out.send_cc(channel_id, parameter, value) {
            error!("MIDI: {e}");
        }
    }

    pub(crate) fn send_clock(&mut self) {
        if let Err(e) = self.midi_out.send_clock() {
            error!("MIDI: {e}");
        }
    }

    fn start(&mut self) {
        self.step = 0;
        if let Err(e) = self.midi_out.send_start() {
            error!("MIDI: {e}");
        }
    }

    /// This function is not intended to be called directly by the user.
    ///
    /// It exists to facilitate code reuse across different environments and platforms.
    pub fn send_continue(&mut self) {
        if let Err(e) = self.midi_out.send_continue() {
            error!("MIDI: {e}");
        }
    }

    pub(crate) fn update(&mut self, next_step: u32) {
        // First send the off signal to every note that end this step.
        let notes = self.play_note_set.remove(&self.step);
        if let Some(notes_off) = notes {
            for n in notes_off {
                if let Err(e) = self
                    .midi_out
                    .send_note_off(n.channel_id, n.midi_note.midi_value())
                {
                    error!("MIDI: {e}");
                }
            }
        };

        // Then play all the notes that were triggered this step...
        for n in &self.notes_to_play {
            if let Err(e) =
                self.midi_out
                    .send_note_on(n.channel_id, n.midi_note.midi_value(), n.midi_note.vel)
            {
                error!("MIDI: {e}");
            }
        }
        // ...and clear them.
        self.notes_to_play.clear();

        // Finally update the step.
        self.step = next_step;
    }

    /// This function is not intended to be called directly by the user.
    ///
    /// It exists to facilitate code reuse across different environments and platforms.
    pub fn stop_all_notes(&mut self) {
        self.start_note_set.iter().for_each(|n| {
            if let Err(e) = self
                .midi_out
                .send_note_off(n.channel_id, n.midi_note.midi_value())
            {
                error!("MIDI: {e}");
            }
        });

        self.play_note_set.values().for_each(|notes| {
            for n in notes {
                if let Err(e) = self
                    .midi_out
                    .send_note_off(n.channel_id, n.midi_note.midi_value())
                {
                    error!("MIDI: {e}");
                }
            }
        });
        self.play_note_set.clear();
    }

    /// This function is not intended to be called directly by the user.
    ///
    /// It exists to facilitate code reuse across different environments and platforms.
    pub fn stop(&mut self) {
        if let Err(e) = self.midi_out.send_stop() {
            error!("MIDI: {e}");
        }
    }

    fn send_message(&mut self, message: MidiMessage) {
        if let Err(e) = self.midi_out.send_message(message) {
            error!("MIDI: {e}");
        }
    }
}
