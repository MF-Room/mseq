#[cfg(feature = "std")]
use std::time::{Duration, Instant};

pub(crate) struct Clock {
    period_us: u64,
    #[cfg(feature = "std")]
    next_clock_timestamp: Instant,
    bpm: u8,
}

#[cfg(feature = "std")]
impl Clock {
    pub(crate) fn new(bpm: u8) -> Self {
        Self {
            period_us: Self::compute_period_us(bpm),
            next_clock_timestamp: Instant::now(),
            bpm,
        }
    }

    pub(crate) fn tick(&mut self) {
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

    pub(crate) fn get_bpm(&self) -> u8 {
        self.bpm
    }
}

#[cfg(not(feature = "std"))]
impl Clock {
    pub(crate) fn new(bpm: u8) -> Self {
        Self {
            period_us: Self::compute_period_us(bpm),
            bpm,
        }
    }

    pub fn tick(&mut self) {
        todo!();
    }

    pub(crate) fn set_bpm(&mut self, bpm: u8) {
        self.bpm = bpm;
        self.period_us = Self::compute_period_us(self.bpm);
    }

    fn compute_period_us(bpm: u8) -> u64 {
        60 * 1000000 / 24 / bpm as u64
    }

    pub fn get_period_us(&self) -> u64 {
        self.period_us
    }

    pub fn get_bpm(&self) -> u8 {
        self.bpm
    }
}
