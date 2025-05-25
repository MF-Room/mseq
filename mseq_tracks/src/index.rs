use crate::TrackError;
use mseq_core::DeteTrack;
use mseq_core::Note;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize, Hash)]
struct Acid {
    name: String,
    file: String,
    root: Note,
    channel: u8,
}
struct Arp {}
struct Div {}

struct Index {
    arp: Vec<Acid>,
}

pub fn load_from_file<P: AsRef<Path> + ToString + Clone>(filename: P) -> Result<(), TrackError> {
    let toml_str = std::fs::read_to_string(filename)?;
    let mut index = toml::from_str(&toml_str)?;
    Ok(())
    /*
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
    */
}
