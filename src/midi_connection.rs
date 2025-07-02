use midir::{Ignore, MidiInput, MidiOutput};
use mseq_core::{InputQueue, MidiController, MidiMessage, MidiOut};
use promptly::{ReadlineError, prompt_default};
use std::{
    collections::VecDeque,
    sync::{Arc, Condvar, Mutex},
};
use thiserror::Error;

const CLOCK: u8 = 0xf8;
const START: u8 = 0xfa;
const CONTINUE: u8 = 0xfb;
const STOP: u8 = 0xfc;
const NOTE_ON: u8 = 0x90;
const NOTE_OFF: u8 = 0x80;
const CC: u8 = 0xB0;
const PC: u8 = 0xC0;

#[derive(Error, Debug)]
pub enum MidiError {
    #[error("Init error: {0}")]
    Init(#[from] midir::InitError),
    #[error("Connect error: {0}")]
    OutConnect(#[from] midir::ConnectError<MidiOutput>),
    #[error("Connect error: {0}")]
    InConnect(#[from] midir::ConnectError<MidiInput>),
    #[error("Send error: {0}")]
    Send(#[from] midir::SendError),
    #[error("Read line [{}: {}]", file!(), line!())]
    ReadLine(#[from] ReadlineError),
    #[error("Invalid port number selected")]
    PortNumber(),
    #[error("No midi output found")]
    NoOutput(),
    #[error("No midi input found")]
    NoInput(),
}

pub struct StdMidiOut(midir::MidiOutputConnection);

impl StdMidiOut {
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

impl MidiOut for StdMidiOut {
    type Error = MidiError;
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
        self.0.send(&[NOTE_ON | (channel_id - 1), note, velocity])?;
        Ok(())
    }

    fn send_note_off(&mut self, channel_id: u8, note: u8) -> Result<(), MidiError> {
        self.0.send(&[NOTE_OFF | (channel_id - 1), note, 0])?;
        Ok(())
    }

    fn send_cc(&mut self, channel_id: u8, parameter: u8, value: u8) -> Result<(), MidiError> {
        self.0.send(&[CC | (channel_id - 1), parameter, value])?;
        Ok(())
    }

    fn send_pc(&mut self, channel_id: u8, value: u8) -> Result<(), MidiError> {
        self.0.send(&[PC | (channel_id - 1), value])?;
        Ok(())
    }
}

/// Struct used to handle the MIDI input. If [`connect`] succeed, an object of type MidiIn
/// is returned.
pub struct StdMidiIn {
    pub connection: midir::MidiInputConnection<(Arc<Mutex<VecDeque<MidiMessage>>>, Arc<Condvar>)>,
    pub queue: Arc<Mutex<VecDeque<MidiMessage>>>,
}

/// MIDI input connection parameters.
#[derive(Clone)]
pub struct MidiInParam {
    /// An enum that is used to specify what kind of MIDI messages should be ignored when receiving messages.
    pub ignore: Ignore,
    /// MIDI port id used to receive the midi messages. If set to `None`, information about the MIDI ports
    /// will be displayed and the input port will be asked to the user with a prompt.
    pub port: Option<u32>,
    pub slave: bool,
}

/// Connect to a specified MIDI input port in order to receive messages.
/// For each (non ignored) incoming MIDI message, the provided callback function will be called.
///The first parameter contains the actual bytes of the MIDI message.
///
///Additional data that should be passed whenever the callback is invoked can be specified by data.
///Use the empty tuple () if you do not want to pass any additional data.
///
///The connection will be kept open as long as the returned MidiInputConnection is kept alive.
/// TODO update doc
pub fn connect(params: MidiInParam, cond_var: Arc<Condvar>) -> Result<StdMidiIn, MidiError> {
    let mut midi_in = MidiInput::new("in")?;
    midi_in.ignore(params.ignore);

    // Find port
    let in_ports = midi_in.ports();

    let in_port = if let Some(p) = params.port {
        match in_ports.get(p as usize) {
            None => return Err(MidiError::PortNumber()),
            Some(x) => x,
        }
    } else {
        match in_ports.len() {
            0 => return Err(MidiError::NoInput()),
            1 => {
                println!(
                    "Choosing the only available intput port: {}",
                    midi_in.port_name(&in_ports[0]).unwrap()
                );
                &in_ports[0]
            }
            _ => {
                println!("\nAvailable input ports:");
                for (i, p) in in_ports.iter().enumerate() {
                    println!("{}: {}", i, midi_in.port_name(p).unwrap());
                }

                let port_number: usize = prompt_default("Select input port", 0)?;
                match in_ports.get(port_number) {
                    None => return Err(MidiError::PortNumber()),
                    Some(x) => x,
                }
            }
        }
    };

    let queue = Arc::new(Mutex::new(InputQueue::new()));
    let input = (queue.clone(), cond_var);
    let conn_in = midi_in.connect(
        in_port,
        "midir-read-input",
        move |_, message, input| {
            let m = MidiMessage::parse(message);
            if let Some(m) = m {
                match m {
                    MidiMessage::Clock => {
                        todo!()
                    }
                    MidiMessage::Start => {
                        todo!()
                    }
                    MidiMessage::Stop => {
                        todo!()
                    }
                    _ => {
                        input.0.lock().unwrap().push_back(m);
                        input.1.notify_all();
                    }
                }
            }
        },
        input,
    )?;

    Ok(StdMidiIn {
        connection: conn_in,
        queue,
    })
}
