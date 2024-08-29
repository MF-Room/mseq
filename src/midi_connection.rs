use midir::MidiOutput;
use promptly::{prompt_default, ReadlineError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MidiError {
    #[error("Init error: {0}")]
    Init(#[from] midir::InitError),
    #[error("Connect error: {0}")]
    Connect(#[from] midir::ConnectError<MidiOutput>),
    #[error("Send error: {0}")]
    Send(#[from] midir::SendError),
    #[error("Read line [{}: {}]", file!(), line!())]
    ReadLine(#[from] ReadlineError),
    #[error("Invalid port number selected")]
    PortNumber(),
    #[error("No midi output found")]
    NoOutput(),
}

const CLOCK: u8 = 0xf8;
const START: u8 = 0xfa;
const CONTINUE: u8 = 0xfb;
const STOP: u8 = 0xfc;
const NOTE_ON: u8 = 0x90;
const NOTE_OFF: u8 = 0x80;
const CC: u8 = 0xB0;

pub trait MidiConnection {
    fn send_start(&mut self) -> Result<(), MidiError>;
    fn send_continue(&mut self) -> Result<(), MidiError>;
    fn send_stop(&mut self) -> Result<(), MidiError>;
    fn send_clock(&mut self) -> Result<(), MidiError>;
    fn send_note_on(&mut self, channel_id: u8, note: u8, velocity: u8) -> Result<(), MidiError>;
    fn send_note_off(&mut self, channel_id: u8, note: u8, velocity: u8) -> Result<(), MidiError>;
    fn send_cc(&mut self, channel_id: u8, parameter: u8, value: u8) -> Result<(), MidiError>;
}

pub struct MidirConnection(midir::MidiOutputConnection);

impl MidirConnection {
    pub(crate) fn new(port: Option<u32>) -> Result<Self, MidiError> {
        let midi_out = MidiOutput::new("out")?;
        let out_ports = midi_out.ports();

        let out_port = if let Some(p) = port {
            match out_ports.get(p as usize) {
                None => return Err(MidiError::PortNumber()),
                Some(x) => x,
            }
        } else {
            match out_ports.len() {
                0 => return Err(MidiError::NoOutput()),
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
                        None => return Err(MidiError::PortNumber()),
                        Some(x) => x,
                    }
                }
            }
        };

        let conn = midi_out.connect(out_port, "output connection")?;
        Ok(Self(conn))
    }
}

impl MidiConnection for MidirConnection {
    fn send_start(&mut self) -> Result<(), MidiError> {
        self.0.send(&[START])?;
        Ok(())
    }

    fn send_continue(&mut self) -> Result<(), MidiError> {
        self.0.send(&[CONTINUE])?;
        Ok(())
    }

    fn send_stop(&mut self) -> Result<(), MidiError> {
        self.0.send(&[STOP])?;
        Ok(())
    }

    fn send_clock(&mut self) -> Result<(), MidiError> {
        self.0.send(&[CLOCK])?;
        Ok(())
    }

    fn send_note_on(&mut self, channel_id: u8, note: u8, velocity: u8) -> Result<(), MidiError> {
        self.0.send(&[NOTE_ON | channel_id, note, velocity])?;
        Ok(())
    }

    fn send_note_off(&mut self, channel_id: u8, note: u8, velocity: u8) -> Result<(), MidiError> {
        self.0.send(&[NOTE_OFF | channel_id, note, velocity])?;
        Ok(())
    }

    fn send_cc(&mut self, channel_id: u8, parameter: u8, value: u8) -> Result<(), MidiError> {
        self.0.send(&[CC | channel_id, parameter, value])?;
        Ok(())
    }
}
