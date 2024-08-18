mod acid;
mod arp;
mod clock;
mod conductor;
mod input;
mod message;
mod midi_controller;
mod note;
mod track;

// Interface
pub use conductor::Conductor;
pub use input::InputManager;
pub use midi_controller::{MidiController, MidiNote};
pub use note::Note;
pub use track::{DeteTrack, Track};

use clock::{clock_gen, compute_period_us};
use midir::{ConnectError, InitError, MidiOutput, MidiOutputConnection};
use promptly::{prompt_default, ReadlineError};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::spawn;
use std::time::Instant;
use thiserror::Error;

pub const RAMPLE_CHANNEL: u8 = 0;
pub const CH_CHANNEL: u8 = 1;
pub const OH_CHANNEL: u8 = 2;
pub const LEAD0_CHANNEL: u8 = 3;
pub const LEAD1_CHANNEL: u8 = 4;
const DEFAULT_BPM: u8 = 120;

#[derive(Error, Debug)]
pub enum TSeqError {
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
}

pub struct Context {
    pub midi: MidiController,
    period_us: u64,
    next_clock_timestamp: Instant,
    step: u32,
    running: bool,
}

impl Context {
    pub fn set_bpm(&mut self, bpm: u8) {
        self.period_us = compute_period_us(bpm);
    }
    pub fn terminate(&mut self) {
        self.running = false
    }
    pub fn get_step(&mut self) -> u32 {
        self.step
    }
}

pub fn run<T: Conductor + Send + 'static>(
    mut conductor: T,
    input_handler: Option<impl InputManager<T> + Send + 'static>,
    port: Option<u32>,
) -> Result<(), TSeqError> {
    let midi_out = MidiOutput::new("out")?;
    let out_ports = midi_out.ports();

    let out_port = if let Some(p) = port {
        match out_ports.get(p as usize) {
            None => return Err(TSeqError::PortNumber()),
            Some(x) => x,
        }
    } else {
        match out_ports.len() {
            0 => return Err(TSeqError::NoOutput()),
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
                    None => return Err(TSeqError::PortNumber()),
                    Some(x) => x,
                }
            }
        }
    };

    let conn = midi_out.connect(out_port, "output connection")?;

    let midi = MidiController::new(conn);

    let mut context = Context {
        midi,
        period_us: compute_period_us(DEFAULT_BPM),
        next_clock_timestamp: Instant::now(),
        step: 0,
        running: true,
    };

    conductor.init(&mut context);

    let context_arc = Arc::new((Mutex::new(context), Condvar::new()));

    // Clock
    let context_arc_1 = context_arc.clone();
    let _ = spawn(move || clock_gen(&context_arc_1));

    let (context, cvar) = &*context_arc;
    let mut context = context.lock().unwrap();

    // Inputs
    let conductor_arc = Arc::new(Mutex::new(conductor));
    if let Some(input) = input_handler {
        let conductor_arc_1 = conductor_arc.clone();
        let _ = spawn(move || input::input_loop(input, &conductor_arc_1));
    }

    let mut running = true;
    while running {
        context = cvar.wait(context).unwrap();
        conductor_arc.lock().unwrap().update(&mut context);
        running = context.running;
    }

    Ok(())
}

pub fn log_send(conn: &mut MidiOutputConnection, message: &[u8]) {
    match conn.send(message) {
        Err(x) => eprintln!("[ERROR] {} (message: {:?})", x, message),
        _ => {}
    }
}
