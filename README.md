# MSeq

[![doc](https://docs.rs/mseq/badge.svg)](https://docs.rs/mseq)
[![crates.io](https://img.shields.io/crates/v/mseq.svg)](https://crates.io/crates/mseq)
[![CI](https://github.com/MF-Room/mseq/actions/workflows/ci.yml/badge.svg)](https://github.com/MF-Room/mseq/actions/workflows/ci.yml)
[![License](https://img.shields.io/github/license/MF-Room/mseq)](https://github.com/MF-Room/mseq/blob/main/LICENSE)

`mseq` is a lightweight MIDI sequencer framework written in Rust. It provides a flexible core for building sequencers that can run in **standalone**, **master**, or **slave** mode, with synchronization over standard MIDI clock and transport messages.

## Features

- Real-time MIDI clock generation and synchronization
- Master/slave transport control with Start/Stop/Continue handling
- Multiple MIDI inputs, each with its own queue and an `input_id` for routing
- Flexible [`Conductor`] trait for defining sequencer logic
- Easy-to-implement tracks via the [`Track`] trait
- Thread-safe, minimal core designed for real-time responsiveness
- Step-based deterministic tracks with [`DeteTrack`]

## Overview

The sequencer is driven by a user-provided [`Conductor`] implementation, which defines how the sequencer initializes, progresses at each clock tick, and reacts to external MIDI messages.

- **No input** → runs standalone with its internal clock and transport, generating MIDI clock and transport messages but ignoring external MIDI input.
- **Master mode** → runs with its internal clock while also processing incoming MIDI events (except for external clock/transport).
- **Slave mode** → synchronizes playback to an external MIDI clock and responds to Start/Stop/Continue messages, dynamically adjusting BPM to match the clock source.

## Conductor Trait

A `Conductor` defines how your sequencer behaves:

- [`Conductor::init`] → called once at startup to initialize state and produce initial [`Instruction`]s (e.g., send program changes or reset messages).
- [`Conductor::update`] → called at every clock tick to advance the sequencer state and emit the instructions for that tick (e.g., note on/off events).
- [`Conductor::handle_input`] → called when a new [`MidiMessage`] arrives, allowing the conductor to react to external inputs in real time. The `input_id` argument (0-based, matching the input's position in the list passed to [`run`]) identifies which input the message came from. It returns a `Vec<Instruction>` processed by the controller: `Instruction::MidiMessage` instructions are forwarded to the output even while paused, while all other instructions are executed only while running.

## MIDI Inputs

[`run`] accepts a `Vec<MidiInParam>`, opening one MIDI input per entry. Each input gets its own queue and is identified by its 0-based position in the list, which is forwarded to [`Conductor::handle_input`] as `input_id`.

- An empty `Vec` runs the sequencer standalone (no input).
- At most one input acts as the clock/transport source: the first one with `slave` set to `true`. Any other `slave` inputs are treated as message-only inputs (a warning is logged).
- With multiple inputs, prefer setting an explicit `port` on each `MidiInParam` rather than leaving it as `None`.

## Tracks

Sequencers can also be built around the [`Track`] trait, which provides a simple interface for describing step-based musical patterns. Each track produces a set of [`Instruction`]s at a given step.

The provided [`DeteTrack`] implements a deterministic looping track:

```rust
use mseq::{Track, DeteTrack, Instruction};

let mut track = DeteTrack::default();
// On each tick, play the instructions for the current step
let instructions: Vec<Instruction> = track.play_step(step);
```
This makes it easy to implement custom track types, from simple step sequencers to more complex algorithmic patterns.

## Usage
The entry point of the crate is the [`run`] function:

```rust
use mseq::{run, Conductor, Context, Instruction, MidiMessage};

struct MyConductor;

impl Conductor for MyConductor {
    fn init(&mut self, _ctx: &mut Context) -> Vec<Instruction> {
        vec![]
    }

    fn update(&mut self, _ctx: &mut Context) -> Vec<Instruction> {
        vec![]
    }

    fn handle_input(&mut self, _input_id: usize, _input: MidiMessage, _ctx: &Context) -> Vec<Instruction> {
        vec![]
    }
}

fn main() -> Result<(), mseq::MSeqError> {
    let conductor = MyConductor;
    let out_port = None;        // Ask the user for the output port
    let midi_in = Vec::new();   // Run standalone (no MIDI input)
    run(conductor, out_port, midi_in)
}
```

## Examples
You can find ready-to-run examples in the [examples](https://github.com/MF-Room/mseq/tree/main/examples) directory. They demonstrate various usage patterns, from simple standalone sequencers to multi-track setups.
