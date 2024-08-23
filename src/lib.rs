mod acid;
mod arp;
mod clock;
mod conductor;
mod midi_controller;
mod note;
mod track;

// Interface
pub use conductor::Conductor;
pub use midi_controller::{MidiController, MidiNote};
pub use note::Note;
pub use track::{DeteTrack, Track};

use clock::Clock;
use midir::{ConnectError, InitError, MidiOutput};
use promptly::{prompt_default, ReadlineError};
use thiserror::Error;

pub const RAMPLE_CHANNEL: u8 = 0;
pub const CH_CHANNEL: u8 = 1;
pub const OH_CHANNEL: u8 = 2;
pub const LEAD0_CHANNEL: u8 = 3;
pub const LEAD1_CHANNEL: u8 = 4;
const DEFAULT_BPM: u8 = 120;

#[derive(Error, Debug)]
pub enum MSeqError {
    #[error("Failed to create a midi output [{}: {}]\n\t{0}", file!(), line!())]
    MidiInit(#[from] InitError),
    #[error("No output found [{}: {}]", file!(), line!())]
    NoOutput(),
    #[error("Read line [{}: {}]", file!(), line!())]
    ReadLine(#[from] ReadlineError),
    #[error("Invalid port number selected [{}: {}]", file!(), line!())]
    PortNumber(),
    #[error("Midi output issue [{}: {}]", file!(), line!())]
    MidiOutput(#[from] ConnectError<MidiOutput>),
    #[error("{0}")]
    Acid(#[from] acid::AcidError),
}

pub struct Context {
    pub midi: MidiController,
    pub(crate) clock: Clock,
    step: u32,
    running: bool,
    on_pause: bool,
}

impl Context {
    pub fn set_bpm(&mut self, bpm: u8) {
        self.clock.set_bpm(bpm);
    }
    pub fn quit(&mut self) {
        self.running = false
    }
    pub fn pause(&mut self) {
        self.on_pause = true;
        self.midi.stop();
    }
    pub fn resume(&mut self) {
        self.on_pause = false;
        self.midi.send_continue();
    }
    pub fn restart(&mut self) {
        self.step = 0;
        self.on_pause = false;
        self.midi.stop();
        self.midi.start();
    }
    pub fn get_step(&mut self) -> u32 {
        self.step
    }
    pub fn run(&mut self, mut conductor: impl Conductor) {
        while self.running {
            conductor.update(self);

            self.clock.tick();
            self.midi.send_clock();

            if !self.on_pause {
                self.step += 1;
                self.midi.update(self.step);
            }
        }
        self.midi.stop();
    }
}

pub fn run(mut conductor: impl Conductor, port: Option<u32>) -> Result<(), MSeqError> {
    let midi_out = MidiOutput::new("out")?;
    let out_ports = midi_out.ports();

    let out_port = if let Some(p) = port {
        match out_ports.get(p as usize) {
            None => return Err(MSeqError::PortNumber()),
            Some(x) => x,
        }
    } else {
        match out_ports.len() {
            0 => return Err(MSeqError::NoOutput()),
            1 => {
                println!(
                    "Choosing the only available output port: {}",
                    midi_out.port_name(&out_ports[0]).unwrap()
                );
                &out_ports[0]
            }
            _ => {
                println!("\nAvailable output ports:");
                for (i, p) in out_ports.iter().enumerate() {
                    println!("{}: {}", i, midi_out.port_name(p).unwrap());
                }

                let port_number: usize = prompt_default("Select output port", 0)?;
                match out_ports.get(port_number) {
                    None => return Err(MSeqError::PortNumber()),
                    Some(x) => x,
                }
            }
        }
    };

    let conn = midi_out.connect(out_port, "output connection")?;

    let midi = MidiController::new(conn);

    let mut ctx = Context {
        midi,
        clock: Clock::new(DEFAULT_BPM),
        step: 0,
        running: true,
        on_pause: true,
    };

    conductor.init(&mut ctx);
    ctx.run(conductor);

    Ok(())
}
