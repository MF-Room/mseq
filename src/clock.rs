use core::time::Duration;
use std::time::Instant;

pub(crate) struct Clock {
    next_clock_timestamp: Instant,
}

impl Clock {
    pub(crate) fn new() -> Self {
        Self {
            next_clock_timestamp: Instant::now(),
        }
    }
    pub(crate) fn tick(&mut self, duration: &Duration) {
        self.next_clock_timestamp += *duration;
        let next_clock_timestamp = self.next_clock_timestamp;

        let sleep_time = next_clock_timestamp - Instant::now();
        spin_sleep::sleep(sleep_time);
    }
}
