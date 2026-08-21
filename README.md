# MSeq

[![doc](https://docs.rs/mseq/badge.svg)](https://docs.rs/mseq)
[![crates.io](https://img.shields.io/crates/v/mseq.svg)](https://crates.io/crates/mseq)
[![CI](https://github.com/MF-Room/mseq/actions/workflows/ci.yml/badge.svg)](https://github.com/MF-Room/mseq/actions/workflows/ci.yml)
[![License](https://img.shields.io/github/license/MF-Room/mseq)](https://github.com/MF-Room/mseq/blob/main/LICENSE)

`mseq` is a lightweight MIDI sequencer framework written in Rust. It provides a flexible core for building sequencers that can run in **standalone**, **master**, or **slave** mode, with synchronization over standard MIDI clock and transport messages.

## Quick Start

Add the crate with `cargo add mseq`, then implement a [`Conductor`] and hand it to [`run`]:

```rust
use mseq::{run, Conductor, Context, Instruction, MidiNote, Note};

struct MyConductor;

impl Conductor for MyConductor {
    fn init(&mut self, ctx: &mut Context) -> Vec<Instruction> {
        ctx.set_bpm(120);
        // The sequencer starts paused: nothing plays until you call start().
        ctx.start();
        vec![]
    }

    fn update(&mut self, ctx: &mut Context) -> Vec<Instruction> {
        // update() runs on every MIDI clock pulse, so there are 24 steps per quarter note.
        if ctx.get_step() % 24 == 0 {
            return vec![Instruction::PlayNote {
                midi_note: MidiNote::new(Note::C, 4, 100),
                len: 12,
                channel_id: 1,
            }];
        }
        vec![]
    }
}

fn main() -> Result<(), mseq::MSeqError> {
    // `None` asks the user to pick an output port, the empty Vec means no MIDI input.
    run(MyConductor, None, Vec::new())
}
```

## Architecture

You implement a [`Conductor`], and optionally one or more [`Track`]s. The engine calls `init` once at
startup, `update` on every MIDI clock pulse, and `handle_input` whenever a MIDI message comes in.
Every call receives a `Context` and returns a `Vec<Instruction>`: control changes and raw messages
are forwarded straight to the output, while notes are played on the step grid with their note-offs
scheduled for you. Transport (`start`, `pause`, `resume`, `quit`) is driven through the `Context`.
This is the whole surface you deal with:

<p align="center">
  <img src="https://raw.githubusercontent.com/MF-Room/mseq/main/docs/architecture.svg"
       alt="mseq_core: what the user implements and what the engine does with it" width="100%">
</p>

## Conductor

A [`Conductor`] defines how your sequencer behaves:

- [`Conductor::init`] is called once at startup, to set up state and emit initial [`Instruction`]s (program changes, reset messages, and so on). Call `ctx.start()` here to leave the initial pause.
- [`Conductor::update`] is called at every clock tick, to advance the sequencer and emit the instructions for that tick.
- [`Conductor::handle_input`] is called when a [`MidiMessage`] arrives, with a 0-based `input_id` telling you which input it came from. While paused, only `Instruction::MidiMessage` is forwarded to the output, and every other instruction is dropped.

How the sequencer is clocked depends on the inputs you give to [`run`]:

- **No input** → runs standalone with its internal clock and transport, generating MIDI clock and transport messages but ignoring external MIDI input.
- **Master mode** → runs with its internal clock while also processing incoming MIDI events (except for external clock/transport).
- **Slave mode** → synchronizes playback to an external MIDI clock and responds to Start/Stop/Continue messages, dynamically adjusting BPM to match the clock source.

## Tracks

Sequencers can also be built around the [`Track`] trait, which describes step-based musical patterns. Each track produces a set of [`Instruction`]s at a given step, so a track is usually played by calling `play_step(ctx.get_step())` from `update` and returning the result.

The provided [`DeteTrack`] implements a deterministic looping track:

```rust
use mseq::{DeteTrack, MidiNote, Note};

// Two notes over 24 steps (one quarter note), looping, on MIDI channel 1.
let track = DeteTrack::new(
    24,
    vec![
        // (note, start step, length in steps)
        (MidiNote::new(Note::A, 4, 89), 0, 12),
        (MidiNote::new(Note::C, 5, 89), 12, 12),
    ],
    Note::A, // Root note, used as the reference for transposition
    1,
    "my_track",
);
```

Implementing [`Track`] yourself is just as easy, from simple step sequencers to more complex algorithmic patterns.

## MIDI Inputs

[`run`] accepts a `Vec<MidiInParam>`, opening one MIDI input per entry. Each input gets its own queue and is identified by its 0-based position in the list, which is forwarded to [`Conductor::handle_input`] as `input_id`.

- An empty `Vec` runs the sequencer standalone (no input).
- At most one input acts as the clock/transport source: the first one with `slave` set to `true`. Any other `slave` inputs are treated as message-only inputs (a warning is logged).
- With multiple inputs, prefer setting an explicit `port` on each `MidiInParam` rather than leaving it as `None`.

## Features

- Real-time MIDI clock generation and synchronization
- Master/slave transport control with Start/Stop/Continue handling
- Multiple MIDI inputs, each with its own queue and an `input_id` for routing
- Flexible [`Conductor`] trait for defining sequencer logic
- Easy-to-implement tracks via the [`Track`] trait
- Thread-safe, minimal core designed for real-time responsiveness
- Step-based deterministic tracks with [`DeteTrack`]

## Examples

You can find ready-to-run examples in the [examples](https://github.com/MF-Room/mseq/tree/main/examples) directory. They demonstrate various usage patterns, from simple standalone sequencers to multi-track setups.

[`Conductor`]: https://docs.rs/mseq/latest/mseq/trait.Conductor.html
[`Conductor::init`]: https://docs.rs/mseq/latest/mseq/trait.Conductor.html#tymethod.init
[`Conductor::update`]: https://docs.rs/mseq/latest/mseq/trait.Conductor.html#tymethod.update
[`Conductor::handle_input`]: https://docs.rs/mseq/latest/mseq/trait.Conductor.html#method.handle_input
[`Track`]: https://docs.rs/mseq/latest/mseq/trait.Track.html
[`DeteTrack`]: https://docs.rs/mseq/latest/mseq/struct.DeteTrack.html
[`Instruction`]: https://docs.rs/mseq/latest/mseq/enum.Instruction.html
[`MidiMessage`]: https://docs.rs/mseq/latest/mseq/enum.MidiMessage.html
[`run`]: https://docs.rs/mseq/latest/mseq/fn.run.html
