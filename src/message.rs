use crate::clock;
use crate::log_send;
use crate::sequence::{control_change, CC_BANK_SEL, CC_KIT_SEL};
use crate::state::{SelPatt, State};
use crate::Channel;
use std::sync::{Arc, Condvar, Mutex};

pub const CLOCK: u8 = 0xf8;
pub const START: u8 = 0xfa;
pub const _CONTINUE: u8 = 0xfb;
pub const STOP: u8 = 0xfc;

pub fn messages_gen(
    channel_arc: &Arc<(Mutex<Channel>, Condvar)>,
    state_arc: &Arc<Mutex<State>>,
    channel_id: u8,
) {
    let (channel, cvar) = &**channel_arc;
    let mut channel = channel.lock().unwrap();
    let mut rng = rand::thread_rng();
    let mut first = true;
    loop {
        channel = cvar.wait(channel).unwrap();

        let mut state = state_arc.lock().unwrap();

        if first {
            log_send(
                &mut channel.conn,
                &control_change(channel_id, CC_KIT_SEL, 0),
            );
            log_send(
                &mut channel.conn,
                &control_change(channel_id, CC_BANK_SEL, 0),
            );

            first = false;
        }

        if !state.running {
            continue;
        }

        if channel.step == 95 {
            log_send(&mut channel.conn, &[START]);
        }

        let (state_data, sel_patt) = state.update(channel.step, &mut channel.conn, &mut rng);
        let sequence = state.get_sequence();

        if let Some(s) = sel_patt {
            // New bpm
            let period = clock::compute_period_us(s.1);
            channel.period_us = period;
            channel.update_timestamp = true;

            // Send message to select next kit
            let message = match s.0 {
                SelPatt::Prev => control_change(channel_id, 100, 0),
                SelPatt::Next => control_change(channel_id, 101, 0),
            };
            log_send(&mut channel.conn, &message);
        }

        sequence.run(channel.step, &mut channel.conn, &mut rng, state_data);
    }
}
