use crate::TrackError;
use crate::arp::ArpDiv;
use crate::{acid, arp, div, midi};
use mseq_core::Note;
use mseq_core::{DeteTrack, MidiNote};
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Default, Deserialize)]
struct Acid {
    name: String,
    file: String,
    root: Note,
    channel: u8,
}

#[derive(Debug, Default, Deserialize)]
struct Arp {
    name: String,
    file: String,
    root: Note,
    channel: u8,
    div: ArpDiv,
}

#[derive(Debug, Default, Deserialize)]
struct Div {
    name: String,
    file: String,
    note: Note,
    octave: u8,
    vel: u8,
    channel: u8,
}

#[derive(Debug, Default, Deserialize)]
struct Midi {
    name: String,
    file: String,
    root: Note,
    channel: u8,
}

#[derive(Debug, Default, Deserialize)]
struct Index {
    acid: Option<Vec<Acid>>,
    arp: Option<Vec<Arp>>,
    div: Option<Vec<Div>>,
    midi: Option<Vec<Midi>>,
}

/// Loads a list of tracks from a TOML index file located at `path`.
///
/// The index file can include references to various track types (`acid`, `arp`, `div`, and `midi`),
/// each with its own configuration and relative file paths. The function resolves the file paths
/// relative to the index file's location, loads the corresponding tracks, and returns them as a list,
/// each paired with its source file path as a string.
///
/// See this [`example`] for a sample index file structure.
///
/// [`example`]: https://github.com/MF-Room/mseq/tree/main/mseq_tracks/tests/res/index.toml
pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Vec<(DeteTrack, String)>, TrackError> {
    let toml_str = std::fs::read_to_string(&path)?;
    let base_dir = path.as_ref().parent().expect("Base path has no parent");
    let index: Index = toml::from_str(&toml_str)?;
    let mut tracks = vec![];

    let mut acid_tracks = index
        .acid
        .unwrap_or_default()
        .iter()
        .map(|a| {
            let relative_path = Path::new(&a.file);
            let path = base_dir.join(relative_path);
            acid::load_from_file(path.clone(), a.root, a.channel, &a.name)
                .map(|t| (t, path.to_string_lossy().into()))
        })
        .collect::<Result<Vec<(DeteTrack, String)>, _>>()?;

    let mut arp_tracks = index
        .arp
        .unwrap_or_default()
        .iter()
        .map(|a| {
            let relative_path = Path::new(&a.file);
            let path = base_dir.join(relative_path);
            arp::load_from_file(path.clone(), a.div, a.root, a.channel, &a.name)
                .map(|t| (t, path.to_string_lossy().into()))
        })
        .collect::<Result<Vec<(DeteTrack, String)>, _>>()?;

    let mut div_tracks = index
        .div
        .unwrap_or_default()
        .iter()
        .map(|a| {
            let relative_path = Path::new(&a.file);
            let path = base_dir.join(relative_path);
            let midi_note = MidiNote {
                note: a.note,
                octave: a.octave,
                vel: a.vel,
            };
            div::load_from_file(path.clone(), midi_note, a.channel, &a.name)
                .map(|t| (t, path.to_string_lossy().into()))
        })
        .collect::<Result<Vec<(DeteTrack, String)>, _>>()?;

    let mut midi_tracks = index
        .midi
        .unwrap_or_default()
        .iter()
        .map(|a| {
            let relative_path = Path::new(&a.file);
            let path = base_dir.join(relative_path);
            midi::load_from_file(path.clone(), a.root, a.channel, &a.name)
                .map(|t| (t, path.to_string_lossy().into()))
        })
        .collect::<Result<Vec<(DeteTrack, String)>, _>>()?;

    tracks.append(&mut acid_tracks);
    tracks.append(&mut arp_tracks);
    tracks.append(&mut div_tracks);
    tracks.append(&mut midi_tracks);

    Ok(tracks)
}
