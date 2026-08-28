# mseq_tracks

[![doc](https://docs.rs/mseq_tracks/badge.svg)](https://docs.rs/mseq_tracks) [![crates.io](https://img.shields.io/crates/v/mseq_tracks.svg)](https://crates.io/crates/mseq_tracks)

Track builders and loaders for the [`mseq`](https://crates.io/crates/mseq) sequencer. This crate
turns high level musical descriptions (acid patterns, arpeggios, clock dividers, MIDI files) into
the `DeteTrack` type that the sequencer plays.

You do not normally depend on it directly: `mseq` re-exports every module below, so
`mseq::acid::load_from_file(..)` works out of the box.

## Track Types

| Module | Produces | Built from |
|---|---|---|
| `acid` | TB-303 style patterns, with rests and ties | `acid::new` or a CSV file |
| `arp` | Arpeggios at a quarter, eighth or sixteenth division | `arp::new` or a CSV file |
| `div` | Clock divider patterns triggering a single note | `div::new` or a CSV file |
| `midi` | A single track MIDI file | `midi::load_from_file` |
| `index` | Several tracks at once | a TOML index file |

Every entry returns a `DeteTrack`, ready to be played from `Conductor::update`.

## Example

```rust
use mseq::{Note, Track};

let mut acid = mseq::acid::load_from_file("examples/res/acid_0.csv", Note::A, 1, "my_acid").unwrap();

// From Conductor::update, play the track at the current step:
let instructions = acid.play_step(ctx.get_step());
```

Sample CSV files live in [examples/res](https://github.com/MF-Room/mseq/tree/main/examples/res),
and a sample index file in
[mseq_tracks/tests/res/index.toml](https://github.com/MF-Room/mseq/blob/main/mseq_tracks/tests/res/index.toml).

## Feature Flags

- `std` (default): file loading and the `TrackError` type. Without it the crate is `no_std`,
  the `midi` and `index` modules disappear, and only the in-memory builders `acid::new`,
  `arp::new` and `div::new` remain.
