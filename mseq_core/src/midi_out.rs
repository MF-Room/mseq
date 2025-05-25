use crate::MidiMessage;

pub(crate) fn is_valid_channel(channel: u8) -> bool {
    (1..=16).contains(&channel)
}

/// This trait is not intended to be implemented by user code.
///
/// It exists to enable code reuse across different MIDI backends through static dispatch.
pub trait MidiOut {
    /// Error type returned by the different member functions.
    type Error: core::fmt::Display;
    /// Send MIDI message
    fn send_message(&mut self, channel_id: u8, message: MidiMessage) -> Result<(), Self::Error> {
        match message {
            MidiMessage::NoteOff { key, vel: _ } => self.send_note_off(channel_id, key),
            MidiMessage::NoteOn { key, vel } => self.send_note_on(channel_id, key, vel),
            MidiMessage::CC { controller, value } => self.send_cc(channel_id, controller, value),
            MidiMessage::Clock => self.send_clock(),
            MidiMessage::Start => self.send_start(),
            MidiMessage::Continue => self.send_continue(),
            MidiMessage::Stop => self.send_stop(),
        }
    }
    /// Send MIDI start message.
    fn send_start(&mut self) -> Result<(), Self::Error>;
    /// Send MIDI continue message.
    fn send_continue(&mut self) -> Result<(), Self::Error>;
    /// Send MIDI stop message.
    fn send_stop(&mut self) -> Result<(), Self::Error>;
    /// Send MIDI clock message.
    fn send_clock(&mut self) -> Result<(), Self::Error>;
    /// Send MIDI note on message.
    fn send_note_on(&mut self, channel_id: u8, note: u8, velocity: u8) -> Result<(), Self::Error>;
    /// Send MIDI note off message.
    fn send_note_off(&mut self, channel_id: u8, note: u8) -> Result<(), Self::Error>;
    /// Send MIDI cc message.
    fn send_cc(&mut self, channel_id: u8, parameter: u8, value: u8) -> Result<(), Self::Error>;
}
