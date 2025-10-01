# mseq_tracks – MSeq Tracks Construction and Loading
This crate provides utilities to define, generate, and load different kinds of
sequencer tracks into the MSeq engine. It serves as a bridge between high-level
musical concepts (e.g., acid patterns, arpeggios, dividers, MIDI files) and
the internal [`DeteTrack`] representation used by the sequencer.
[`DeteTrack`]: mseq_core::DeteTrack