# MSeq

[![doc](https://docs.rs/mseq/badge.svg)](https://docs.rs/mseq) [![crates.io](https://img.shields.io/crates/v/mseq.svg)](https://crates.io/crates/mseq) [![CI/CD](https://github.com/MF-Room/mseq/actions/workflows/rust.yml/badge.svg)](https://github.com/MF-Room/mseq/actions/workflows/rust.yml)

`mseq` is a lightweight MIDI sequencer framework written in Rust. It provides a flexible core for building sequencers that can run in **standalone**, **master**, or **slave** mode, with synchronization over standard MIDI clock and transport messages.

## Features

- Real-time MIDI clock generation and synchronization  
- Master/slave transport control with Start/Stop/Continue handling  
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
- [`Conductor::handle_input`] → called when a new [`MidiMessage`] arrives, allowing the conductor to react to external inputs in real time.  

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

    fn handle_input(&mut self, input: MidiMessage, _ctx: &Context) -> Vec<Instruction> {
        vec![]
    }
}

fn main() -> Result<(), mseq::MSeqError> {
    let conductor = MyConductor;
    let out_port = None;
    let midi_in = None;
    run(conductor, out_port, midi_in)
}
```

## Examples
You can find ready-to-run examples in the [examples](https://github.com/MF-Room/mseq/tree/main/examples) directory. directory. They demonstrate various usage patterns, from simple standalone sequencers to multi-track setups.
