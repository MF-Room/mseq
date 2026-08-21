# mseq_core

[![doc](https://docs.rs/mseq_core/badge.svg)](https://docs.rs/mseq_core) [![crates.io](https://img.shields.io/crates/v/mseq_core.svg)](https://crates.io/crates/mseq_core)

Core framework for building custom MIDI sequencers.
`mseq_core` provides the foundational traits and utilities needed to implement
your own MIDI sequencer, with a focus on portability and modularity.
This crate is built with `#![no_std]`, making it suitable for embedded platforms
as well as standard operating systems.

It is the portable engine only: it has no MIDI I/O and no run loop of its own, so it is always
used through a platform layer. On a desktop OS, use [`mseq`](https://crates.io/crates/mseq),
which wraps this crate and provides both.

## Getting Started

To create a custom sequencer, you typically:

- Implement the `Conductor` trait to define your sequencer's control logic.
  The sequencer starts paused, so call `context.start()` from `init` to get it playing.
- Define one or more tracks by either:
  - Implementing the `Track` trait for custom behavior.
  - Instantiating `DeteTrack` for deterministic, looping patterns.

## Architecture

You implement a `Conductor`, and optionally one or more `Track`s. The engine calls `init` once at
startup, `update` on every MIDI clock pulse, and `handle_input` whenever a MIDI message comes in.
Every call receives a `Context` and returns a `Vec<Instruction>`: control changes and raw messages
are forwarded straight to the output, while notes are played on the step grid with their note-offs
scheduled for you. Transport (`start`, `pause`, `resume`, `quit`) is driven through the `Context`.
This is the whole surface you deal with:

<p align="center">
  <img src="https://raw.githubusercontent.com/MF-Room/mseq/main/docs/architecture.svg"
       alt="mseq_core: what the user implements and what the engine does with it" width="100%">
</p>

## Platform Support

- For OS-based systems, use the [`mseq`](https://crates.io/crates/mseq) crate, a reference implementation of `mseq_core` for standard platforms.
- For embedded development (e.g., STM32F4), see the [`mseq_embedded`](https://github.com/MF-Room/mseq_embedded) repository, which provides an STM32-specific integration of `mseq_core`.

## Crate Features

- No `std` dependency (`#![no_std]` compatible).
- Modular and extensible design.
- Reusable across multiple platforms.
