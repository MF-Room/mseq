pub(crate) fn is_valid_channel(channel: u8) -> bool {
    (1..=16).contains(&channel)
}

/// This trait should not be implemented in the user code. The purpose of this trait is be able to reuse
/// the same code with different midi API, using static dispatch.
pub trait MidiOut {
    type Error: core::fmt::Display;
    fn send_start(&mut self) -> Result<(), Self::Error>;
    fn send_continue(&mut self) -> Result<(), Self::Error>;
    fn send_stop(&mut self) -> Result<(), Self::Error>;
    fn send_clock(&mut self) -> Result<(), Self::Error>;
    fn send_note_on(&mut self, channel_id: u8, note: u8, velocity: u8) -> Result<(), Self::Error>;
    fn send_note_off(&mut self, channel_id: u8, note: u8) -> Result<(), Self::Error>;
    fn send_cc(&mut self, channel_id: u8, parameter: u8, value: u8) -> Result<(), Self::Error>;
}
