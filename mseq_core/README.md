# mseq_core

Core framework for building custom MIDI sequencers.
`mseq_core` provides the foundational traits and utilities needed to implement
your own MIDI sequencer, with a focus on portability and modularity.
This crate is built with `#![no_std]`, making it suitable for embedded platforms
as well as standard operating systems.
## Getting Started
To create a custom sequencer, you typically:
- Implement the [`Conductor`] trait to define your sequencer's control logic.
- Define one or more tracks by either:
  - Implementing the [`Track`] trait for custom behavior.
  - Instantiating [`DeteTrack`] for deterministic, looping patterns.
## Platform Support
- For OS-based systems, use the [`mseq`](https://crates.io/crates/mseq) crate — a reference implementation of `mseq_core` for standard platforms.
- For embedded development (e.g., STM32F4), see the [`mseq_embedded`](https://github.com/MF-Room/mseq_embedded) repository, which provides an STM32-specific integration of `mseq_core`.
## Crate Features
- No `std` dependency (`#![no_std]` compatible).
- Modular and extensible design.
- Reusable across multiple platforms.