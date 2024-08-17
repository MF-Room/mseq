mod acid;
mod arp;
mod clock;
mod conductor;
mod message;

use std::thread::sleep;
use std::time::{Duration, Instant};
mod midi_controller;
mod note;
mod track;
//pub use acid::{Acid, AcidLead, Timing};
//pub use arp::{Arp, ArpDiv, ArpLead};
use clock::{clock_gen, compute_period_us};
use midi_controller::{MidiController, MidiNote};
//use message::messages_gen;
use midir::{ConnectError, InitError, MidiOutput, MidiOutputConnection};
use promptly::{prompt, prompt_default, ReadlineError};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::spawn;
use thiserror::Error;

pub const RAMPLE_CHANNEL: u8 = 0;
pub const CH_CHANNEL: u8 = 1;
pub const OH_CHANNEL: u8 = 2;
pub const LEAD0_CHANNEL: u8 = 3;
pub const LEAD1_CHANNEL: u8 = 4;

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

struct Channel {
    midi: MidiController,
    period_us: u64,
    timestamp: Instant,
    update_timestamp: bool,
    bpm_step: u32,
    step: u32,
}

pub struct Step {}

pub fn run(channel_id: u8, port: Option<u32>) -> Result<(), TSeqError> {
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

    // TODO: replace 156 by bpm
    let channel = Channel {
        midi,
        period_us: compute_period_us(120),
        timestamp: Instant::now(),
        update_timestamp: true,
        bpm_step: 0,
        step: 0,
    };
    let channel_arc = Arc::new((Mutex::new(channel), Condvar::new()));

    /*
        let mut info = Info {
            root: patterns[0].root,
            bpm: patterns[0].bpm,
            lead0: (Lead0State::None, None),
            lead1: (Lead1State::None, None),
            scale: Scale::default(),
        };
    */

    // Clock
    let channel_arc_1 = channel_arc.clone();
    let _ = spawn(move || clock_gen(&channel_arc_1));

    loop {
        let mut t = channel_arc.0.lock().unwrap();
        let note = MidiNote::new(note::Note::C, 4, 127);
        t.midi.play_note(note, 10, 3);
        let sleep_time = Duration::from_secs(1);
        drop(t);
        sleep(sleep_time);
    }

    // Messages
    /*
        let channel_arc_1 = channel_arc.clone();
        let _ = spawn(move || messages_gen(&channel_arc_1, &state_arc_1, channel_id - 1));

        loop {
            let lead0 = match info.lead0.1 {
                Some(s) => format!("{}({})", info.lead0.0, s),
                None => format!("{}", info.lead0.0),
            };
            let lead1 = match info.lead1.1 {
                Some(s) => format!("{}({})", info.lead1.0, s),
                None => format!("{}", info.lead1.0),
            };

            let s = format!(
                "[{} {} | {} | {} | {}]",
                info.root.get_str(),
                info.bpm,
                lead0,
                lead1,
                info.scale
            );

            let s: String = prompt(s)?;
            if let Some(i) = handle(&s, &channel_arc, &state_arc) {
                info = i;
            } else {
                break;
            }
        }
    */
    Ok(())
}

pub fn log_send(conn: &mut MidiOutputConnection, message: &[u8]) {
    match conn.send(message) {
        Err(x) => eprintln!("[ERROR] {} (message: {:?})", x, message),
        _ => {}
    }
}
