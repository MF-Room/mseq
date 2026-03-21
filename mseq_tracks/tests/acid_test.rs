use mseq_core::{MidiNote, Note, Track};
use mseq_tracks::acid::{AcidTrig, Timing, new};

#[test]
fn acid_track_basic_pattern() {
    // Create a simple acid pattern: Note, Rest, Note, Rest
    let pattern = vec![
        AcidTrig {
            midi_note: MidiNote::new(Note::C, 3, 100),
            timing: Timing::Note,
        },
        AcidTrig {
            midi_note: MidiNote::new(Note::C, 3, 100),
            timing: Timing::Rest,
        },
        AcidTrig {
            midi_note: MidiNote::new(Note::D, 3, 100),
            timing: Timing::Note,
        },
        AcidTrig {
            midi_note: MidiNote::new(Note::D, 3, 100),
            timing: Timing::Rest,
        },
    ];

    let track = new(pattern, Note::C, 1, "test_acid");

    // Verify the track has the correct properties
    assert_eq!(track.get_name(), "test_acid");

    // Check that notes are scheduled at the correct steps
    let notes_at_step_0 = track.get_notes_start_at_step(0);
    assert_eq!(notes_at_step_0.len(), 1);
    assert_eq!(notes_at_step_0[0].0.note, Note::C);
    assert_eq!(notes_at_step_0[0].0.octave, 3);
    assert_eq!(notes_at_step_0[0].1, 3); // 3 ticks duration

    let notes_at_step_12 = track.get_notes_start_at_step(12);
    assert_eq!(notes_at_step_12.len(), 1);
    assert_eq!(notes_at_step_12[0].0.note, Note::D);
    assert_eq!(notes_at_step_12[0].0.octave, 3);
    assert_eq!(notes_at_step_12[0].1, 3); // 3 ticks duration

    // Verify no notes at rest positions
    let notes_at_step_6 = track.get_notes_start_at_step(6);
    assert!(notes_at_step_6.is_empty());

    let notes_at_step_18 = track.get_notes_start_at_step(18);
    assert!(notes_at_step_18.is_empty());
}

#[test]
fn acid_track_with_ties() {
    // Create a pattern with ties: Note, Tie, Tie, Rest
    let pattern = vec![
        AcidTrig {
            midi_note: MidiNote::new(Note::C, 3, 100),
            timing: Timing::Note,
        },
        AcidTrig {
            midi_note: MidiNote::new(Note::C, 3, 100),
            timing: Timing::Tie,
        },
        AcidTrig {
            midi_note: MidiNote::new(Note::C, 3, 100),
            timing: Timing::Tie,
        },
        AcidTrig {
            midi_note: MidiNote::new(Note::C, 3, 100),
            timing: Timing::Rest,
        },
    ];

    let track = new(pattern, Note::C, 1, "test_acid_ties");

    // Verify the track has the correct properties
    assert_eq!(track.get_name(), "test_acid_ties");

    // Check that the first note starts at step 0 with extended duration due to ties
    let notes_at_step_0 = track.get_notes_start_at_step(0);
    assert_eq!(notes_at_step_0.len(), 1);
    assert_eq!(notes_at_step_0[0].0.note, Note::C);
    assert_eq!(notes_at_step_0[0].0.octave, 3);
    assert_eq!(notes_at_step_0[0].1, 15); // 3 + 6*2 (two ties) = 15 ticks duration

    // Verify no notes at rest position
    let notes_at_step_18 = track.get_notes_start_at_step(18);
    assert!(notes_at_step_18.is_empty());
}

#[test]
fn acid_track_with_ties_different_notes() {
    // Create a pattern with ties using different notes: Note, Tie (same), Tie (different), Rest
    let pattern = vec![
        AcidTrig {
            midi_note: MidiNote::new(Note::C, 3, 100),
            timing: Timing::Note,
        },
        AcidTrig {
            midi_note: MidiNote::new(Note::C, 3, 100),
            timing: Timing::Tie,
        },
        AcidTrig {
            midi_note: MidiNote::new(Note::D, 3, 100),
            timing: Timing::Tie,
        },
        AcidTrig {
            midi_note: MidiNote::new(Note::C, 3, 100),
            timing: Timing::Rest,
        },
    ];

    let track = new(pattern, Note::C, 1, "test_acid_ties_diff");

    // Verify the track has the correct properties
    assert_eq!(track.get_name(), "test_acid_ties_diff");

    // Check that the first note (C3) starts at step 0 with duration extended by one tie
    let notes_at_step_0 = track.get_notes_start_at_step(0);
    assert_eq!(notes_at_step_0.len(), 1);
    assert_eq!(notes_at_step_0[0].0.note, Note::C);
    assert_eq!(notes_at_step_0[0].0.octave, 3);
    assert_eq!(notes_at_step_0[0].1, 13); // 7 + 6*1 (one tie, different note follows) = 13 ticks duration

    // Check that the second note (D3) starts at step 12 with normal duration (no ties)
    let notes_at_step_12 = track.get_notes_start_at_step(12);
    assert_eq!(notes_at_step_12.len(), 1);
    assert_eq!(notes_at_step_12[0].0.note, Note::D);
    assert_eq!(notes_at_step_12[0].0.octave, 3);
    assert_eq!(notes_at_step_12[0].1, 3); // 3 ticks duration (no ties)

    // Verify no notes at rest position
    let notes_at_step_18 = track.get_notes_start_at_step(18);
    assert!(notes_at_step_18.is_empty());
}
