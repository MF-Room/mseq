pub mod acid;
pub mod arp;
pub mod div;
pub mod midi;

extern crate alloc;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum TrackError {
    /// Error type related to CSV file parsing
    #[cfg(feature = "std")]
    #[error("Failed to parse csv file [{f}: {l}]\n\t{0}", f=file!(), l=line!())]
    Reading(#[from] csv::Error),
    #[cfg(feature = "std")]
    #[error("Failed to read midi file: {0}")]
    Io(#[from] std::io::Error),
    #[cfg(feature = "std")]
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
