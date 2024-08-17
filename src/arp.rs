use std::usize;

use crate::scale::Scale;
use crate::trig::Trig;
use crate::LEAD0_CHANNEL;
use rand::seq::SliceRandom;
use rand::thread_rng;

const LEAD_VEL: u8 = 100;

#[derive(Default, Clone)]
pub enum ArpDiv {
    #[default]
    T4,
    T8,
    T16,
}

#[derive(Default, Clone)]
pub struct ArpLead {
    pattern: Vec<Vec<(u8, i8)>>,
    scales: Vec<Scale>,
    arp_div: ArpDiv,
    played: bool,
    name: String,
}

pub struct Arp {
    patterns: Vec<ArpLead>,
    cur_id: usize,
    cur_sub_id: u8,
    prev_note: u8,
}

impl ArpLead {
    pub fn new(
        pattern: Vec<Vec<(u8, i8)>>,
        arp_div: ArpDiv,
        scales: Vec<Scale>,
        name: &str,
    ) -> Self {
        let len = pattern.len();
        Self {
            pattern,
            scales,
            arp_div,
            played: false,
            name: String::from(format!("{}({})", name, len)),
        }
    }
}

impl Arp {
    pub fn get_name(&self) -> String {
        self.patterns[self.cur_id].name.clone()
    }
    pub fn new(patterns: Vec<ArpLead>) -> Self {
        //Check that we have at least one pattern for each scale
        let mut sc: [bool; 3] = [false; 3];
        for p in &patterns {
            for s in &p.scales {
                match s {
                    Scale::NaturalMinor => sc[0] = true,
                    Scale::HarmonicMinor => sc[1] = true,
                    Scale::PhrygianMode => sc[2] = true,
                }
            }
        }

        for (i, s) in sc.iter().enumerate() {
            if !s {
                match i {
                    0 => panic!("No arp lead for Natural Minor Scale!"),
                    1 => panic!("No arp lead for Harmonic Minor Scale!"),
                    _ => panic!("No arp lead for Phrygian Mode!"),
                }
            }
        }

        let mut patterns = patterns.clone();
        patterns.shuffle(&mut thread_rng());
        Self {
            patterns,
            cur_id: 0,
            cur_sub_id: 0,
            prev_note: 0,
        }
    }

    pub fn get_trig(&mut self, step: u32, root: u8) -> Vec<Trig> {
        let pattern = &self.patterns[self.cur_id];
        let arp_div = &pattern.arp_div;
        let div = match arp_div {
            ArpDiv::T4 => 24,
            ArpDiv::T8 => 12,
            ArpDiv::T16 => 6,
        };

        if step % div == 0 {
            let pattern = &pattern.pattern[self.cur_sub_id as usize];
            let t = step / div;
            let cur_trig = t as usize % pattern.len();
            let cur_note = &pattern[cur_trig];
            let note = root + cur_note.0 + (cur_note.1 * 12) as u8;
            self.prev_note = note;
            vec![Trig {
                start_end: true,
                channel_id: LEAD0_CHANNEL,
                note,
                velocity: LEAD_VEL,
            }]
        } else if step % div == (div / 2) {
            vec![Trig {
                start_end: false,
                channel_id: LEAD0_CHANNEL,
                note: self.prev_note,
                velocity: LEAD_VEL,
            }]
        } else {
            vec![]
        }
    }

    pub fn toggle_sub(&mut self) {
        let pattern = &self.patterns[self.cur_id];
        let len = pattern.pattern.len();
        self.cur_sub_id = (self.cur_sub_id + 1) % len as u8;
    }

    pub fn get_prev_note(&self) -> (u8, u8) {
        (self.prev_note, LEAD_VEL)
    }

    pub fn next_pattern(&mut self, scale: Scale) {
        let len = self.patterns.len();
        let mut next_id = None;
        for i in 0..len {
            let pattern = &mut self.patterns[i];
            if !pattern.played && pattern.scales.contains(&scale) {
                next_id = Some(i);
                break;
            }
        }
        self.cur_id = if let Some(id) = next_id {
            id
        } else {
            let mut next_id = None;
            for i in 0..len {
                let pattern = &mut self.patterns[i];
                pattern.played = false;
                if let None = next_id {
                    if pattern.scales.contains(&scale) {
                        next_id = Some(i);
                    }
                }
            }

            match next_id {
                Some(id) => id,
                None => panic!("No Arp lead in {}", scale),
            }
        };

        self.cur_sub_id = 0;
        let pattern = &mut self.patterns[self.cur_id];
        pattern.played = true;
    }
}
