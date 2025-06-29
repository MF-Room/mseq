use crate::MidiNote;

/// Midi Message representation according to the Midi Standard
/// https://www.music.mcgill.ca/~ich/classes/mumt306/StandardMIDIfileformat.html#BMA1_
#[derive(PartialEq)]
pub enum MidiMessage {
    /// Note Off event. This message is sent when a note is released.
    NoteOff { note: MidiNote },
    /// Note On event. This message is sent when a note is pressed.
    NoteOn { note: MidiNote },
    /// Control Change. This message is sent when a controller value changes.
    CC {
        /// Controller number
        controller: u8,
        /// The new value
        value: u8,
    },
    /// Timing Clock. Sent 24 times per quarter note when synchronisation is required.
    Clock,
    /// Start. Start the current sequence playing.
    Start,
    /// Continue. Continue at the point the sequence was Stopped.
    Continue,
    /// Stop. Stop the current sequence.
    Stop,
}
