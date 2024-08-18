use std::time::Duration;
use std::time::Instant;

pub(crate) struct Clock {
    period_us: u64,
    next_clock_timestamp: Instant,
    bpm: u8,
}

impl Clock {
    pub(crate) fn new(bpm: u8) -> Self {
        Self {
            period_us: Self::compute_period_us(bpm),
            next_clock_timestamp: Instant::now(),
            bpm,
        }
    }

    pub fn tick(&mut self) {
        self.next_clock_timestamp += Duration::from_micros(self.period_us);
        let next_clock_timestamp = self.next_clock_timestamp;

        let sleep_time = next_clock_timestamp - Instant::now();
        spin_sleep::sleep(sleep_time);
    }

    pub(crate) fn set_bpm(&mut self, bpm: u8) {
        self.bpm = bpm;
        self.period_us = Self::compute_period_us(self.bpm);
    }

    fn compute_period_us(bpm: u8) -> u64 {
        60 * 1000000 / 24 / bpm as u64
    }
}
