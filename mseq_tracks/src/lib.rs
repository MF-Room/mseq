use mseq_core::{DeteTrack, Note};
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
    #[error("Track file is empty")]
    Emptyfile,
    #[error("Invalid header in track file")]
    BadHeader,
    #[error("Invalid header in track file")]
    ParseInt(#[from] ParseIntError),
}

use std::num::ParseIntError;
#[cfg(feature = "std")]
use std::path::Path;

#[cfg(feature = "std")]
pub fn load_from_file<P: AsRef<Path> + ToString + Clone>(
    filename: P,
) -> Result<DeteTrack, TrackError> {
    use arp::ArpDiv;

    let mut rdr = csv::Reader::from_path(filename.clone())?;
    let record = rdr.records().next().ok_or(TrackError::Emptyfile)??;
    if record.len() < 3 {
        return Err(TrackError::BadHeader);
    }
    let root = if record[1].eq("A") {
        Note::A
    } else if record[1].eq("AS") {
        Note::AS
    } else if record[1].eq("B") {
        Note::B
    } else if record[1].eq("C") {
        Note::C
    } else if record[1].eq("CS") {
        Note::CS
    } else if record[1].eq("D") {
        Note::D
    } else if record[1].eq("DS") {
        Note::DS
    } else if record[1].eq("E") {
        Note::E
    } else if record[1].eq("F") {
        Note::F
    } else if record[1].eq("FS") {
        Note::FS
    } else if record[1].eq("G") {
        Note::G
    } else if record[1].eq("GS") {
        Note::GS
    } else {
        return Err(TrackError::BadHeader);
    };

    let channel_id = record[2].parse()?;
    let name = filename.to_string();

    if record[0].eq("acid") {
        acid::load_from_reader(&mut rdr, root, channel_id, &name)
    } else if record[0].eq("arp") {
        if record.len() < 4 {
            return Err(TrackError::BadHeader);
        }
        let div = if record[2].eq("T4") {
            ArpDiv::T4
        } else if record[2].eq("T8") {
            ArpDiv::T8
        } else if record[2].eq("T16") {
            ArpDiv::T16
        } else {
            return Err(TrackError::BadHeader);
        };
        arp::load_from_reader(&mut rdr, div, root, channel_id, &name)
    } else {
        return Err(TrackError::BadHeader);
    }
}
