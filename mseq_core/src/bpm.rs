pub(crate) struct Bpm {
    period_us: u64,
    bpm: Option<u8>,
}

impl Bpm {
    pub(crate) fn new(bpm: u8) -> Self {
        Self {
            period_us: Self::compute_period_us(bpm),
            bpm: Some(bpm),
        }
    }

    pub(crate) fn set_bpm(&mut self, bpm: u8) {
        self.bpm = Some(bpm);
        self.period_us = Self::compute_period_us(bpm);
    }

    /// Marks the bpm as unknown. The period of the last known bpm is kept.
    pub(crate) fn clear(&mut self) {
        self.bpm = None;
    }

    fn compute_period_us(bpm: u8) -> u64 {
        // Guard against a 0 bpm, which would divide by zero.
        60 * 1000000 / 24 / bpm.max(1) as u64
    }

    pub(crate) fn get_period_us(&self) -> u64 {
        self.period_us
    }

    pub(crate) fn get_bpm(&self) -> Option<u8> {
        self.bpm
    }
}
