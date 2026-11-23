#![cfg(feature = "get-size")]

use get_size2::GetSize;
use mseq_core::{DeteTrack, MidiNote, Note};

#[test]
fn notes_have_no_heap_part() {
    assert_eq!(Note::A.get_heap_size(), 0);
    assert_eq!(MidiNote::new(Note::A, 3, 100).get_heap_size(), 0);
}

#[test]
fn a_track_counts_its_notes_and_its_name() {
    let note = MidiNote::new(Note::A, 3, 100);
    let notes = vec![(note, 0, 6); 4];
    // `vec![x; n]` allocates exactly `n` elements, and `DeteTrack::new` keeps that Vec.
    assert_eq!(notes.capacity(), 4);
    let track = DeteTrack::new(24, notes, Note::A, 1, "acid");
    assert_eq!(
        track.get_heap_size(),
        4 * size_of::<(MidiNote, u32, u32)>() + "acid".len()
    );
}
