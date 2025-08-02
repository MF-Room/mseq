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

#[cfg(feature = "std")]
/// The `midi` module provides tools for generating and loading MIDI
/// tracks into MSeq.
pub mod midi;
extern crate alloc;

#[cfg(feature = "std")]
/// The `index` module provides functionality for loading multiple track types
/// into the MSeq sequencer from a unified `.toml` index file.
pub mod index;

#[cfg(feature = "std")]
use thiserror::Error;

/// An error type representing issues that may occur when loading or generating
/// tracks.
#[cfg(feature = "std")]
#[derive(Error, Debug)]
pub enum TrackError {
    /// Error type related to CSV file parsing
    #[cfg(feature = "std")]
    #[error("Failed to parse csv file [{f}: {l}]\n\t{0}", f=file!(), l=line!())]
    Reading(#[from] csv::Error),
    /// I/O error while reading from file
    #[cfg(feature = "std")]
    #[error("Failed to read file: {0}")]
    Io(#[from] std::io::Error),
    /// MIDI parsing error from `midly
    #[cfg(feature = "std")]
    #[error("Midly error: {0}")]
    Midly(#[from] midly::Error),
    /// Error indicating that a note was started multiple times without being released (MIDI files only).
    #[cfg(feature = "std")]
    #[error("Cannot play the same note before it ends")]
    DuplicateNote,
    /// Error raised when a note-off event occurs for a note that was never started (MIDI files only).
    #[cfg(feature = "std")]
    #[error("Cannot end a note before playing it")]
    WrongNoteOff,
    /// Error raised when the MIDI file does not contain exactly one track.
    #[cfg(feature = "std")]
    #[error("Midi file doesn't contain exactly one track")]
    BadFormat,
    /// Error raised when the MIDI file contains an unsuported timing specification.
    #[cfg(feature = "std")]
    #[error("Unsupported timing specification")]
    BadTiming,
    /// Error type related to TOML file parsing.
    #[cfg(feature = "std")]
    #[error("Failed to deserialize Toml: {0}")]
    Toml(#[from] toml::de::Error),
}
