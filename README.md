# MSeq

[![doc](https://docs.rs/mseq/badge.svg)](https://docs.rs/mseq) [![crates.io](https://img.shields.io/crates/v/mseq.svg)](https://crates.io/crates/mseq) [![CI/CD](https://github.com/MF-Room/mseq/actions/workflows/rust.yml/badge.svg)](https://github.com/MF-Room/mseq/actions/workflows/rust.yml)

Library for developing MIDI Sequencers.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
mseq = "0.1"
```
To start using `mseq`, create a struct that implements the `Conductor` trait.

You can then add tracks to your sequencer by adding fields (to your struct that implements the
`Conductor` trait) of type `DeteTrack` or more generally fields that implement the trait `Track`.

Once this is done, you can play your track in the `Conductor::update` function of your struct that
implements the `Conductor` trait. To do so, call the method `MidiController::play_track` (of
the `Context::midi`) with the track you want to play as a parameter.

You can find some examples in the [`examples`](https://github.com/MF-Room/mseq/tree/main/examples) directory.
