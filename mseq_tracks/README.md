# mseq_tracks – MSeq Tracks Construction and Loading

[![doc](https://docs.rs/mseq_tracks/badge.svg)](https://docs.rs/mseq_tracks) [![crates.io](https://img.shields.io/crates/v/mseq_tracks.svg)](https://crates.io/crates/mseq_tracks)

This crate provides utilities to define, generate, and load different kinds of
sequencer tracks into the [`MSeq`](https://crates.io/crates/mseq) engine. It serves as a bridge between high-level
musical concepts (e.g., acid patterns, arpeggios, dividers, MIDI files) and
the internal `DeteTrack` representation used by the sequencer.
