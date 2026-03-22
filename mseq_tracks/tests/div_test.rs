use mseq_core::{MidiNote, Note, Track};
use mseq_tracks::div::{self, ClockDiv};

#[test]
fn div_track_basic_pattern() {
    // Create a simple division pattern: trigger every 12 clocks for 48 clocks
    let pattern = vec![ClockDiv {
        div: 12,
        duration: 48,
    }];

    let note = MidiNote::new(Note::C, 3, 100);
    let track = div::new(pattern, note, 1, "test_div_basic");

    // Verify the track has the correct properties
    assert_eq!(track.get_name(), "test_div_basic");

    // With div=12, duration=48: nb_trigs = 48/12 = 4 triggers
    // Triggers at: 0, 12, 24, 36 (each with duration 12)
    let notes_at_step_0 = track.get_notes_start_at_step(0);
    assert_eq!(notes_at_step_0.len(), 1);
    assert_eq!(notes_at_step_0[0].0.note, Note::C);
    assert_eq!(notes_at_step_0[0].0.octave, 3);
    assert_eq!(notes_at_step_0[0].1, 12);

    let notes_at_step_12 = track.get_notes_start_at_step(12);
    assert_eq!(notes_at_step_12.len(), 1);
    assert_eq!(notes_at_step_12[0].1, 12);

    let notes_at_step_24 = track.get_notes_start_at_step(24);
    assert_eq!(notes_at_step_24.len(), 1);
    assert_eq!(notes_at_step_24[0].1, 12);

    let notes_at_step_36 = track.get_notes_start_at_step(36);
    assert_eq!(notes_at_step_36.len(), 1);
    assert_eq!(notes_at_step_36[0].1, 12);

    // No notes at positions that aren't multiples of 12
    let notes_at_step_6 = track.get_notes_start_at_step(6);
    assert!(notes_at_step_6.is_empty());

    // At step 48 (track length), the pattern wraps around
    let notes_at_step_48 = track.get_notes_start_at_step(48);
    assert_eq!(notes_at_step_48.len(), 1);
    assert_eq!(notes_at_step_48[0].1, 12);
}

#[test]
fn div_track_multiple_patterns() {
    // Create a pattern with multiple clock divisions
    let pattern = vec![
        ClockDiv {
            div: 6,
            duration: 24,
        }, // 24/6 = 4 triggers at 0,6,12,18
        ClockDiv {
            div: 12,
            duration: 24,
        }, // 24/12 = 2 triggers at 24,36
    ];

    let note = MidiNote::new(Note::E, 2, 80);
    let track = div::new(pattern, note, 1, "test_div_multi");

    // Verify the track has the correct properties
    assert_eq!(track.get_name(), "test_div_multi");

    // First pattern: div=6, duration=24
    let notes_at_step_0 = track.get_notes_start_at_step(0);
    assert_eq!(notes_at_step_0.len(), 1);
    assert_eq!(notes_at_step_0[0].1, 6);

    let notes_at_step_6 = track.get_notes_start_at_step(6);
    assert_eq!(notes_at_step_6.len(), 1);
    assert_eq!(notes_at_step_6[0].1, 6);

    let notes_at_step_12 = track.get_notes_start_at_step(12);
    assert_eq!(notes_at_step_12.len(), 1);
    assert_eq!(notes_at_step_12[0].1, 6);

    let notes_at_step_18 = track.get_notes_start_at_step(18);
    assert_eq!(notes_at_step_18.len(), 1);
    assert_eq!(notes_at_step_18[0].1, 6);

    // Second pattern: div=12, duration=24 (starts at step 24)
    let notes_at_step_24 = track.get_notes_start_at_step(24);
    assert_eq!(notes_at_step_24.len(), 1);
    assert_eq!(notes_at_step_24[0].1, 12);

    let notes_at_step_36 = track.get_notes_start_at_step(36);
    assert_eq!(notes_at_step_36.len(), 1);
    assert_eq!(notes_at_step_36[0].1, 12);

    // No notes in between patterns
    let notes_at_step_20 = track.get_notes_start_at_step(20);
    assert!(notes_at_step_20.is_empty());

    // Total length should be 24 + 24 = 48, pattern wraps around at 48
    let notes_at_step_48 = track.get_notes_start_at_step(48);
    assert_eq!(notes_at_step_48.len(), 1);
    assert_eq!(notes_at_step_48[0].1, 6); // First pattern repeats
}

#[test]
fn div_track_edge_cases() {
    // Test edge case: div equals duration (only one trigger)
    let pattern = vec![ClockDiv {
        div: 24,
        duration: 24,
    }];

    let note = MidiNote::new(Note::G, 4, 60);
    let track = div::new(pattern, note, 1, "test_div_edge");

    // Should have exactly one trigger at step 0
    let notes_at_step_0 = track.get_notes_start_at_step(0);
    assert_eq!(notes_at_step_0.len(), 1);
    assert_eq!(notes_at_step_0[0].1, 24);

    // At step 24 (track length), the pattern wraps around
    let notes_at_step_24 = track.get_notes_start_at_step(24);
    assert_eq!(notes_at_step_24.len(), 1);
    assert_eq!(notes_at_step_24[0].1, 24);
}

#[test]
fn div_track_complex_pattern() {
    // Test a more complex pattern similar to the example in res/div.csv
    let pattern = vec![
        ClockDiv {
            div: 12,
            duration: 48,
        }, // 4 triggers
        ClockDiv {
            div: 6,
            duration: 48,
        }, // 8 triggers
        ClockDiv {
            div: 24,
            duration: 48,
        }, // 2 triggers
    ];

    let note = MidiNote::new(Note::A, 3, 100);
    let track = div::new(pattern, note, 1, "test_div_complex");

    // Verify first pattern (div=12, duration=48)
    let notes_at_step_0 = track.get_notes_start_at_step(0);
    assert_eq!(notes_at_step_0.len(), 1);
    assert_eq!(notes_at_step_0[0].1, 12);

    let notes_at_step_36 = track.get_notes_start_at_step(36);
    assert_eq!(notes_at_step_36.len(), 1);
    assert_eq!(notes_at_step_36[0].1, 12);

    // Verify second pattern (div=6, duration=48) starts at step 48
    let notes_at_step_48 = track.get_notes_start_at_step(48);
    assert_eq!(notes_at_step_48.len(), 1);
    assert_eq!(notes_at_step_48[0].1, 6);

    let notes_at_step_54 = track.get_notes_start_at_step(54);
    assert_eq!(notes_at_step_54.len(), 1);
    assert_eq!(notes_at_step_54[0].1, 6);

    let notes_at_step_90 = track.get_notes_start_at_step(90);
    assert_eq!(notes_at_step_90.len(), 1);
    assert_eq!(notes_at_step_90[0].1, 6);

    // Verify third pattern (div=24, duration=48) starts at step 96
    let notes_at_step_96 = track.get_notes_start_at_step(96);
    assert_eq!(notes_at_step_96.len(), 1);
    assert_eq!(notes_at_step_96[0].1, 24);

    let notes_at_step_120 = track.get_notes_start_at_step(120);
    assert_eq!(notes_at_step_120.len(), 1);
    assert_eq!(notes_at_step_120[0].1, 24);

    // Total length should be 48 + 48 + 48 = 144, pattern wraps around at 144
    let notes_at_step_144 = track.get_notes_start_at_step(144);
    assert_eq!(notes_at_step_144.len(), 1);
    assert_eq!(notes_at_step_144[0].1, 12); // First pattern repeats
}
