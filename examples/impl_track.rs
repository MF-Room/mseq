use mseq::{Conductor, Instruction, MidiNote, Track};
use rand::{Rng, distributions::Uniform, thread_rng};

struct MyTrack {
    channel_id: u8,
}

// Implement a track for full freedom (randomization, automatization...)
impl Track for MyTrack {
    fn play_step(&mut self, step: u32) -> Vec<Instruction> {
        // Midi channel id to send the note to
        if step % 8 == 0 {
            // Choose a random note
            let note = MidiNote {
                note: thread_rng().sample(Uniform::new(0u8, 11u8)).into(),
                octave: 4,
                vel: 127,
            };

            // Note length in number of steps
            let note_length = 3;

            // Request to play the note to the midi controller.
            vec![Instruction::PlayNote {
                midi_note: note,
                len: note_length,
                channel_id: self.channel_id,
            }]
        } else {
            vec![]
        }
    }
}

struct MyConductor {
    track: MyTrack,
}

impl Conductor for MyConductor {
    fn init(&mut self, context: &mut mseq::Context) -> Vec<Instruction> {
        // The sequencer is on pause by default
        context.start();
        vec![]
    }

    fn update(&mut self, context: &mut mseq::Context) -> Vec<Instruction> {
        let step = context.get_step();

        // Quit after 960 steps
        if step == 959 {
            context.quit();
            return vec![];
        }
        // The conductor plays the track
        self.track.play_step(step)
    }
}

fn main() {
    env_logger::init();

    if let Err(e) = mseq::run(
        MyConductor {
            track: MyTrack { channel_id: 0 },
        },
        // The midi port will be selected at runtime by the user
        None,
        None,
    ) {
        println!("An error occured: {:?}", e);
    }
}
