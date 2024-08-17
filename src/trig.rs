use crate::log_send;
use crate::sequence::{end_note, start_note};
use midir::MidiOutputConnection;

pub struct Trig {
    pub start_end: bool,
    pub channel_id: u8,
    pub note: u8,
    pub velocity: u8,
}

pub fn trigger(conn: &mut MidiOutputConnection, trigs: &Vec<Trig>) {
    for t in trigs {
        trigger_single(conn, t)
    }
}

pub fn trigger_single(conn: &mut MidiOutputConnection, t: &Trig) {
    if t.start_end {
        log_send(conn, &start_note(t.channel_id, t.note, t.velocity));
    } else {
        log_send(conn, &end_note(t.channel_id, t.note, t.velocity));
    }
}
