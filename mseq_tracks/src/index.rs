use crate::TrackError;
use crate::acid;
use mseq_core::DeteTrack;
use mseq_core::Note;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Default, Deserialize)]
struct Acid {
    name: String,
    file: String,
    root: Note,
    channel: u8,
}

#[derive(Debug, Default, Deserialize)]
struct Arp {}

#[derive(Debug, Default, Deserialize)]
struct Div {}

#[derive(Debug, Default, Deserialize)]
struct Index {
    acid: Vec<Acid>,
}

pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Vec<DeteTrack>, TrackError> {
    let toml_str = std::fs::read_to_string(&path)?;
    let base_dir = path.as_ref().parent().expect("Base path has no parent");
    let index: Index = toml::from_str(&toml_str)?;
    let tracks = index
        .acid
        .into_iter()
        .map(|a| {
            let relative_path = Path::new(&a.file);
            let path = base_dir.join(relative_path);
            acid::load_from_file(path, a.root, a.channel, &a.name)
        })
        .collect::<Result<_, _>>()?;
    Ok(tracks)
}
