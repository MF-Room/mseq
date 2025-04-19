pub(crate) struct Bpm {
    period_us: u64,
    bpm: u8,
}

impl Bpm {
    pub(crate) fn new(bpm: u8) -> Self {
        Self {
            period_us: Self::compute_period_us(bpm),
            bpm,
        }
    }

    pub(crate) fn set_bpm(&mut self, bpm: u8) {
        self.bpm = bpm;
        self.period_us = Self::compute_period_us(self.bpm);
    }

    fn compute_period_us(bpm: u8) -> u64 {
        60 * 1000000 / 24 / bpm as u64
    }

    pub(crate) fn get_period_us(&self) -> u64 {
        self.period_us
    }

    pub(crate) fn get_bpm(&self) -> u8 {
        self.bpm
    }
}
