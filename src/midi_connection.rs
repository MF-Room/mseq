#[cfg(feature = "std")]
include!("./std_midi_connection.rs");

#[cfg(not(feature = "std"))]
include!("./no_std_midi_connection.rs");

#[cfg(not(feature = "std"))]
use crate::no_std_mod::*;

pub(crate) fn is_valid_channel(channel: u8) -> bool {
    (1..=16).contains(&channel)
}

/// This trait should not be implemented in the user code. The purpose of this trait is be able to reuse
/// the same code with different midi API, using static dispatch.
pub trait MidiOut {
    #[cfg(feature = "std")]
    #[doc(hidden)]
    fn send_start(&mut self) -> Result<(), MidiError>;
    #[cfg(not(feature = "std"))]
    fn send_start(&mut self) -> Result<(), MidiError>;
    #[cfg(feature = "std")]
    #[doc(hidden)]
    fn send_continue(&mut self) -> Result<(), MidiError>;
    #[cfg(not(feature = "std"))]
    fn send_continue(&mut self) -> Result<(), MidiError>;
    #[cfg(feature = "std")]
    #[doc(hidden)]
    fn send_stop(&mut self) -> Result<(), MidiError>;
    #[cfg(not(feature = "std"))]
    fn send_stop(&mut self) -> Result<(), MidiError>;
    #[cfg(feature = "std")]
    #[doc(hidden)]
    fn send_clock(&mut self) -> Result<(), MidiError>;
    #[cfg(not(feature = "std"))]
    fn send_clock(&mut self) -> Result<(), MidiError>;
    #[cfg(feature = "std")]
    #[doc(hidden)]
    fn send_note_on(&mut self, channel_id: u8, note: u8, velocity: u8) -> Result<(), MidiError>;
    #[cfg(not(feature = "std"))]
    fn send_note_on(&mut self, channel_id: u8, note: u8, velocity: u8) -> Result<(), MidiError>;
    #[cfg(feature = "std")]
    #[doc(hidden)]
    fn send_note_off(&mut self, channel_id: u8, note: u8) -> Result<(), MidiError>;
    #[cfg(not(feature = "std"))]
    fn send_note_off(&mut self, channel_id: u8, note: u8) -> Result<(), MidiError>;
    #[cfg(feature = "std")]
    #[doc(hidden)]
    fn send_cc(&mut self, channel_id: u8, parameter: u8, value: u8) -> Result<(), MidiError>;
    #[cfg(not(feature = "std"))]
    fn send_cc(&mut self, channel_id: u8, parameter: u8, value: u8) -> Result<(), MidiError>;
}
