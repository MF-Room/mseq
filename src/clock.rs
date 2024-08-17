use crate::log_send;
use crate::message;
use crate::Channel;
use std::mem::drop;
use std::sync::{Arc, Condvar, Mutex};
use std::thread::sleep;
use std::time::{Duration, Instant};

pub fn clock_gen(channel_arc: &Arc<(Mutex<Channel>, Condvar)>) {
    loop {
        let (channel, cvar) = &**channel_arc;
        let mut channel = channel.lock().unwrap();
        log_send(&mut channel.conn, &[message::CLOCK]);

        if channel.update_timestamp {
            channel.update_timestamp = false;
            channel.timestamp = Instant::now();
            channel.bpm_step = 0;
        }

        let period = channel.period_us;
        channel.step += 1;
        channel.bpm_step += 1;
        let bpm_step = channel.bpm_step;
        let timestamp = channel.timestamp.clone();

        // Unlock the mutex
        drop(channel);
        cvar.notify_one();
        let sleep_time = Duration::from_micros(bpm_step as u64 * period) - timestamp.elapsed();
        sleep(sleep_time);
    }
}

pub fn compute_period_us(bpm: u8) -> u64 {
    60 * 1000000 / 24 / bpm as u64
}
