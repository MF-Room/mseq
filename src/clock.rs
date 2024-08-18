use crate::log_send;
use crate::Context;
use std::mem::drop;
use std::process::exit;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;
use std::time::Instant;

pub const CLOCK: u8 = 0xf8;

pub fn clock_gen(context_arc: &Arc<(Mutex<Context>, Condvar)>) {
    loop {
        let (context, cvar) = &**context_arc;
        let mut context = context.lock().unwrap();
        log_send(&mut context.midi.conn, &[CLOCK]);

        let step = context.step;
        context.midi.update(step);

        context.next_clock_timestamp =
            context.next_clock_timestamp + Duration::from_micros(context.period_us);
        context.step += 1;
        let next_clock_timestamp = context.next_clock_timestamp;

        // Unlock the mutex
        drop(context);
        cvar.notify_one();
        let sleep_time = next_clock_timestamp - Instant::now();

        spin_sleep::sleep(sleep_time);
    }
}

pub fn compute_period_us(bpm: u8) -> u64 {
    60 * 1000000 / 24 / bpm as u64
}
