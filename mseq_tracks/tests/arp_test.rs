use mseq_core::{MidiNote, Note, Track};
use mseq_tracks::arp::{self, ArpDiv};

#[test]
fn arp_track_basic_pattern() {
    // Create a simple arpeggio pattern: C3, E3, G3, C4
    let pattern = vec![
        MidiNote::new(Note::C, 3, 100),
        MidiNote::new(Note::E, 3, 100),
        MidiNote::new(Note::G, 3, 100),
        MidiNote::new(Note::C, 4, 100),
    ];

    let track = arp::new(pattern, ArpDiv::T8, Note::C, 1, "test_arp");

    // Verify the track has the correct properties
    assert_eq!(track.get_name(), "test_arp");

    // Check that notes are scheduled at the correct steps for T8 division (12 ticks per step)
    let notes_at_step_0 = track.get_notes_start_at_step(0);
    assert_eq!(notes_at_step_0.len(), 1);
    assert_eq!(notes_at_step_0[0].0.note, Note::C);
    assert_eq!(notes_at_step_0[0].0.octave, 3);
    assert_eq!(notes_at_step_0[0].1, 6); // T8 division: factor/2 = 12/2 = 6 ticks duration

    let notes_at_step_12 = track.get_notes_start_at_step(12);
    assert_eq!(notes_at_step_12.len(), 1);
    assert_eq!(notes_at_step_12[0].0.note, Note::E);
    assert_eq!(notes_at_step_12[0].0.octave, 3);
    assert_eq!(notes_at_step_12[0].1, 6);

    let notes_at_step_24 = track.get_notes_start_at_step(24);
    assert_eq!(notes_at_step_24.len(), 1);
    assert_eq!(notes_at_step_24[0].0.note, Note::G);
    assert_eq!(notes_at_step_24[0].0.octave, 3);
    assert_eq!(notes_at_step_24[0].1, 6);

    let notes_at_step_36 = track.get_notes_start_at_step(36);
    assert_eq!(notes_at_step_36.len(), 1);
    assert_eq!(notes_at_step_36[0].0.note, Note::C);
    assert_eq!(notes_at_step_36[0].0.octave, 4);
    assert_eq!(notes_at_step_36[0].1, 6);
}

#[test]
fn arp_track_different_divisions() {
    // Test different time divisions with the same pattern
    let pattern = vec![
        MidiNote::new(Note::A, 3, 100),
        MidiNote::new(Note::A, 4, 100),
    ];

    // T4 division (quarter notes) - 24 ticks per step
    let track_t4 = arp::new(pattern.clone(), ArpDiv::T4, Note::A, 1, "test_arp_t4");
    assert_eq!(track_t4.get_name(), "test_arp_t4");
    
    let notes_t4 = track_t4.get_notes_start_at_step(0);
    assert_eq!(notes_t4.len(), 1);
    assert_eq!(notes_t4[0].1, 12); // T4 division: factor/2 = 24/2 = 12 ticks duration

    // T8 division (eighth notes) - 12 ticks per step
    let track_t8 = arp::new(pattern.clone(), ArpDiv::T8, Note::A, 1, "test_arp_t8");
    assert_eq!(track_t8.get_name(), "test_arp_t8");
    
    let notes_t8 = track_t8.get_notes_start_at_step(0);
    assert_eq!(notes_t8.len(), 1);
    assert_eq!(notes_t8[0].1, 6); // T8 division: factor/2 = 12/2 = 6 ticks duration

    // T16 division (sixteenth notes) - 6 ticks per step
    let track_t16 = arp::new(pattern, ArpDiv::T16, Note::A, 1, "test_arp_t16");
    assert_eq!(track_t16.get_name(), "test_arp_t16");
    
    let notes_t16 = track_t16.get_notes_start_at_step(0);
    assert_eq!(notes_t16.len(), 1);
    assert_eq!(notes_t16[0].1, 3); // T16 division: factor/2 = 6/2 = 3 ticks duration
}

#[test]
fn arp_track_single_note() {
    // Test with single note pattern
    let pattern = vec![
        MidiNote::new(Note::F, 4, 80),
    ];

    let track = arp::new(pattern, ArpDiv::T16, Note::C, 1, "test_arp_single");

    // Verify the track has the correct properties
    assert_eq!(track.get_name(), "test_arp_single");

    // Check that the single note is scheduled correctly for T16 division
    let notes_at_step_0 = track.get_notes_start_at_step(0);
    assert_eq!(notes_at_step_0.len(), 1);
    assert_eq!(notes_at_step_0[0].0.note, Note::F);
    assert_eq!(notes_at_step_0[0].0.octave, 4);
    assert_eq!(notes_at_step_0[0].1, 3); // T16 division: factor/2 = 6/2 = 3 ticks duration

    // For a single note with T16 division:
    // - factor = 6, so track length = 1 * 6 = 6 ticks
    // - The note starts at step 0 with duration 3, so it plays from 0-3
    // - Due to wrapping, the same note appears again at step 6 (which is equivalent to step 0 in the next cycle)
    let notes_at_step_6 = track.get_notes_start_at_step(6);
    assert_eq!(notes_at_step_6.len(), 1);
    assert_eq!(notes_at_step_6[0].0.note, Note::F);
    assert_eq!(notes_at_step_6[0].0.octave, 4);
    assert_eq!(notes_at_step_6[0].1, 3);
}