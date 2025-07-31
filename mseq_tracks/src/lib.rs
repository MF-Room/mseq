//! # `track` – MSeq Track Construction and Loading
//!
//! This crate provides utilities to define, generate, and load different kinds of
//! sequencer tracks into the MSeq engine. It serves as a bridge between high-level
//! musical concepts (e.g., acid patterns, arpeggios, dividers, MIDI files) and
//! the internal [`DeteTrack`] representation used by the sequencer.
//!
//! [`DeteTrack`]: mseq_core::DeteTrack

#![warn(missing_docs)]

/// The `acid` module provides tools for generating and loading acid-style
/// tracks into MSeq.
pub mod acid;
/// The `arp` module provides tools for generating and loading arp-style
/// tracks into MSeq.
pub mod arp;
/// The `div` module provides tools for generating and loading clock divider-style
/// tracks into MSeq.
pub mod div;
/// The `midi` module provides tools for generating and loading MIDI
/// tracks into MSeq.
pub mod midi;
extern crate alloc;

#[cfg(feature = "std")]
/// The `index` module provides functionality for loading multiple track types
/// into the MSeq sequencer from a unified `.toml` index file.
pub mod index;

use thiserror::Error;

/// An error type representing issues that may occur when loading or generating
/// tracks.
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
    #[cfg(feature = "std")]
    #[error("Failed to deserialize Toml: {0}")]
    Toml(#[from] toml::de::Error),
}
