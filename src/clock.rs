use crate::log_send;
use crate::Context;
use std::mem::drop;
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

pub const CLOCK: u8 = 0xf8;

pub fn clock_gen(context_arc: &Arc<(Mutex<Context>, Condvar)>) {
    loop {
        let (context, cvar) = &**context_arc;
        let mut context = context.lock().unwrap();
        let step = context.step;
        log_send(&mut context.midi.conn, &[CLOCK]);
        context.midi.update(step);

        if context.update_timestamp {
            context.update_timestamp = false;
            context.timestamp = Instant::now();
            context.bpm_step = 0;
        }

        let period = context.period_us;
        context.step += 1;
        context.bpm_step += 1;
        let bpm_step = context.bpm_step;
        let timestamp = context.timestamp.clone();


        // Unlock the mutex
        drop(context);
        cvar.notify_one();
        let sleep_time = Duration::from_micros(bpm_step as u64 * period) - timestamp.elapsed();
        spin_sleep::sleep(sleep_time);
    }
}

pub fn compute_period_us(bpm: u8) -> u64 {
    60 * 1000000 / 24 / bpm as u64
}
