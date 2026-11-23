#![cfg(feature = "std")]

use mseq_core::{DeteTrack, Instruction, MidiNote, Note, Track};
use mseq_tracks::arp::ArpDiv;
use mseq_tracks::{acid, arp, div, midi};

/// Plays 4 bars of `track` and checks that it keeps its name and plays notes, all on `channel`.
fn check_track(mut track: DeteTrack, name: &str, channel: u8) {
    assert_eq!(track.get_name(), name);

    let channels: Vec<u8> = (0..384)
        .flat_map(|step| track.play_step(step))
        .map(|instruction| match instruction {
            Instruction::PlayNote { channel_id, .. } => channel_id,
            other => panic!("Unexpected instruction: {other:?}"),
        })
        .collect();
    assert!(!channels.is_empty(), "Track {name} plays no note");
    assert!(channels.iter().all(|&c| c == channel));
}

#[test]
fn load_acid_from_file() {
    let track = acid::load_from_file("tests/res/acid.csv", Note::C, 7, "acid").unwrap();
    check_track(track, "acid", 7);
}

#[test]
fn load_arp_from_file() {
    let track = arp::load_from_file("tests/res/arp.csv", ArpDiv::T8, Note::A, 6, "arp").unwrap();
    check_track(track, "arp", 6);
}

#[test]
fn load_div_from_file() {
    let note = MidiNote::new(Note::C, 3, 63);
    let track = div::load_from_file("tests/res/div.csv", note, 5, "div").unwrap();
    check_track(track, "div", 5);
}

#[test]
fn load_midi_from_file() {
    let track = midi::load_from_file("tests/res/midi.mid", Note::A, 4, "midi").unwrap();
    check_track(track, "midi", 4);
}
