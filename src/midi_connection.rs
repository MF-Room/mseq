use crate::Ignore;
use midir::{MidiInput, MidiOutput};
use midly::{live::LiveEvent, MidiMessage};
use promptly::ReadlineError;
#[cfg(not(feature = "embedded"))]
use std::sync::{Arc, Mutex};

#[cfg(feature = "embedded")]
use crate::embedded_mod::*;
#[cfg(not(feature = "embedded"))]
use {promptly::prompt_default, thiserror::Error};

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

#[cfg(not(feature = "embedded"))]
const CLOCK: u8 = 0xf8;
#[cfg(not(feature = "embedded"))]
const START: u8 = 0xfa;
#[cfg(not(feature = "embedded"))]
const CONTINUE: u8 = 0xfb;
#[cfg(not(feature = "embedded"))]
const STOP: u8 = 0xfc;
#[cfg(not(feature = "embedded"))]
const NOTE_ON: u8 = 0x90;
#[cfg(not(feature = "embedded"))]
const NOTE_OFF: u8 = 0x80;
const CC: u8 = 0xB0;

pub(crate) fn is_valid_channel(channel: u8) -> bool {
    (1..=16).contains(&channel)
}

/// This trait should not be implemented in the user code. The purpose of this trait is be able to reuse
/// the same code with different midi API, using static dispatch.
pub trait MidiOut {
    #[cfg(not(feature = "embedded"))]
    #[doc(hidden)]
    fn send_start(&mut self) -> Result<(), MidiError>;
    #[cfg(feature = "embedded")]
    fn send_start(&mut self) -> Result<(), MidiError>;
    #[cfg(not(feature = "embedded"))]
    #[doc(hidden)]
    fn send_continue(&mut self) -> Result<(), MidiError>;
    #[cfg(feature = "embedded")]
    fn send_continue(&mut self) -> Result<(), MidiError>;
    #[cfg(not(feature = "embedded"))]
    #[doc(hidden)]
    fn send_stop(&mut self) -> Result<(), MidiError>;
    #[cfg(feature = "embedded")]
    fn send_stop(&mut self) -> Result<(), MidiError>;
    #[cfg(not(feature = "embedded"))]
    #[doc(hidden)]
    fn send_clock(&mut self) -> Result<(), MidiError>;
    #[cfg(feature = "embedded")]
    fn send_clock(&mut self) -> Result<(), MidiError>;
    #[cfg(not(feature = "embedded"))]
    #[doc(hidden)]
    fn send_note_on(&mut self, channel_id: u8, note: u8, velocity: u8) -> Result<(), MidiError>;
    #[cfg(feature = "embedded")]
    fn send_note_on(&mut self, channel_id: u8, note: u8, velocity: u8) -> Result<(), MidiError>;
    #[cfg(not(feature = "embedded"))]
    #[doc(hidden)]
    fn send_note_off(&mut self, channel_id: u8, note: u8) -> Result<(), MidiError>;
    #[cfg(feature = "embedded")]
    fn send_note_off(&mut self, channel_id: u8, note: u8) -> Result<(), MidiError>;
    #[cfg(not(feature = "embedded"))]
    #[doc(hidden)]
    fn send_cc(&mut self, channel_id: u8, parameter: u8, value: u8) -> Result<(), MidiError>;
    #[cfg(feature = "embedded")]
    fn send_cc(&mut self, channel_id: u8, parameter: u8, value: u8) -> Result<(), MidiError>;
}

#[cfg(not(feature = "embedded"))]
pub struct MidirOut(midir::MidiOutputConnection);

#[cfg(not(feature = "embedded"))]
impl MidirOut {
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

#[cfg(not(feature = "embedded"))]
impl MidiOut for MidirOut {
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
}

fn parse(message: &[u8]) -> Option<(u8, MidiMessage)> {
    if let Ok(LiveEvent::Midi { channel, message }) = LiveEvent::parse(message) {
        Some((channel.as_int(), message))
    } else {
        None
    }
}

pub trait MidiIn {
    fn handle(&self, channel: u8, message: &MidiMessage);
}

/// Struct used to handle the MIDI input. If [`MidiIn::connect`] succeed, an object of type MidiIn
/// is returned.
#[cfg(not(feature = "embedded"))]
#[allow(dead_code)]
pub struct MidirIn<T: 'static> {
    connection: midir::MidiInputConnection<Arc<Mutex<T>>>,
    data: Arc<Mutex<T>>,
}

/// MIDI input connection parameters.
pub struct MidiInParam {
    /// An enum that is used to specify what kind of MIDI messages should be ignored when receiving messages.
    pub ignore: Ignore,
    /// MIDI port id used to receive the midi messages. If set to `None`, information about the MIDI ports
    /// will be displayed and the input port will be asked to the user with a prompt.
    pub port: Option<u32>,
}

/// Connect to a specified MIDI input port in order to receive messages.
/// For each (non ignored) incoming MIDI message, the provided callback function will be called.
///The first parameter contains the actual bytes of the MIDI message.
///
///Additional data that should be passed whenever the callback is invoked can be specified by data.
///Use the empty tuple () if you do not want to pass any additional data.
///
///The connection will be kept open as long as the returned MidiInputConnection is kept alive.
#[cfg(not(feature = "embedded"))]
pub fn connect<T: MidiIn + Send>(handler: T, params: MidiInParam) -> Result<MidirIn<T>, MidiError> {
    let mut midi_in = MidiInput::new("in")?;
    midi_in.ignore(params.ignore);
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

                let port_number: usize = prompt_default("Select output port", 0)?;
                match in_ports.get(port_number) {
                    None => return Err(MidiError::PortNumber()),
                    Some(x) => x,
                }
            }
        }
    };

    let data = Arc::new(Mutex::new(handler));

    let conn_in = midi_in.connect(
        in_port,
        "midir-read-input",
        move |_, message, data| {
            let m = parse(message);
            if let Some(m) = m {
                data.lock().unwrap().handle(m.0, &m.1);
            }
        },
        data.clone(),
    )?;

    Ok(MidirIn {
        connection: conn_in,
        data,
    })
}
