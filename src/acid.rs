/*
use crate::trig::Trig;
use crate::LEAD1_CHANNEL;
use rand::seq::SliceRandom;
use rand::thread_rng;

#[derive(Default, Clone, Copy)]
pub enum Timing {
    Note,
    Tie,
    #[default]
    Rest,
}

use Timing::*;

#[derive(Default, Copy, Clone)]
pub struct AcidTrig {
    note: (u8, i8),
    vel: u8,
    slide: bool,
    timing: Timing,
}

#[derive(Default, Clone)]
pub struct AcidLead {
    pattern: Vec<AcidTrig>,
    scales: Vec<Scale>,
    played: bool,
    name: String,
}

#[derive(Default)]
pub struct Acid {
    patterns: Vec<AcidLead>,
    cur_id: usize,
    prev_note: (Timing, u8, u8),
}

impl AcidLead {
    pub fn new(pattern: Vec<((u8, i8), u8, bool, Timing)>, scales: Vec<Scale>, name: &str) -> Self {
        let pattern = pattern
            .iter()
            .map(|u| AcidTrig {
                note: u.0,
                vel: u.1,
                slide: u.2,
                timing: u.3,
            })
            .collect();

        Self {
            pattern,
            scales,
            played: false,
            name: String::from(name),
        }
    }
}

impl Acid {
    pub fn get_name(&self) -> String {
        self.patterns[self.cur_id].name.clone()
    }
    pub fn new(patterns: Vec<AcidLead>) -> Self {
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
                    0 => panic!("No acid lead for Natural Minor Scale!"),
                    1 => panic!("No acid lead for Harmonic Minor Scale!"),
                    _ => panic!("No acid lead for Phrygian Mode!"),
                }
            }
        }

        let mut patterns = patterns.clone();
        patterns.shuffle(&mut thread_rng());
        Self {
            patterns,
            cur_id: 0,
            prev_note: (Rest, 0, 0),
        }
    }

    pub fn get_trig(&mut self, step: u32, root: u8) -> Vec<Trig> {
        let mut res = vec![];
        let pattern = &self.patterns[self.cur_id].pattern;
        if step % 6 == 0 {
            let t = step / 6;
            let cur_trig = t as usize % pattern.len();
            self.prev_note.0 = pattern[cur_trig].timing;
            let cur_note = &pattern[cur_trig];

            let no_end = if let Tie = cur_note.timing {
                true
            } else {
                false
            };

            let prev_note = self.prev_note;

            let note = root + cur_note.note.0 + (cur_note.note.1 * 12) as u8;
            match cur_note.timing {
                Note => {
                    res.push(Trig {
                        start_end: true,
                        channel_id: LEAD1_CHANNEL,
                        note,
                        velocity: cur_note.vel,
                    });
                    self.prev_note.1 = note;
                    self.prev_note.2 = cur_note.vel;
                }
                _ => {}
            }

            // When there is a slide end the note after the previous one started
            if !no_end && cur_note.slide {
                match prev_note.0 {
                    Note | Tie => res.push(Trig {
                        start_end: false,
                        channel_id: LEAD1_CHANNEL,
                        note: prev_note.1,
                        velocity: prev_note.2,
                    }),
                    _ => {}
                }
            }
        } else if step % 6 == 3 {
            let t = (step + 3) / 6;
            let cur_trig = t as usize % pattern.len();
            self.prev_note.0 = pattern[cur_trig].timing;
            let cur_note = &pattern[cur_trig];

            let no_end = if let Tie = cur_note.timing {
                true
            } else {
                false
            };

            if !no_end && !cur_note.slide {
                match self.prev_note.0 {
                    Note | Tie => res.push(Trig {
                        start_end: false,
                        channel_id: LEAD1_CHANNEL,
                        note: self.prev_note.1,
                        velocity: self.prev_note.2,
                    }),

                    _ => {}
                }
            }
        }
        res
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
                None => panic!("No Acid line in {}", scale),
            }
        };

        let pattern = &mut self.patterns[self.cur_id];
        pattern.played = true;
    }

    pub fn get_prev_note(&self) -> (u8, u8) {
        (self.prev_note.1, self.prev_note.2)
    }
}
*/
