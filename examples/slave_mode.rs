use mseq::{Conductor, Instruction, MidiInParam, MidiNote};

struct MyConductor {}

impl Conductor for MyConductor {
    fn init(&mut self, context: &mut mseq::Context) -> Vec<Instruction> {
        // The sequencer is on pause by default
        context.start();
        vec![]
    }

    fn update(&mut self, context: &mut mseq::Context) -> Vec<Instruction> {
        let step = context.get_step();
        if context.get_step() == 959 {
            context.quit();
        }

        let mut ins = vec![];
        if step % 24 == 0 {
            println!("BPM: {}", context.get_bpm());

            // Simple 4 on the floor
            let note = MidiNote {
                note: mseq::Note::C,
                octave: 4,
                vel: 127,
            };

            ins.push(Instruction::PlayNote {
                midi_note: note,
                len: 6,
                channel_id: 1,
            });
        }
        ins
    }
}

fn main() {
    env_logger::init();

    let midi_in_param = MidiInParam {
        ignore: mseq::Ignore::None,
        port: None,
        slave: true,
    };
    if let Err(e) = mseq::run(
        MyConductor {},
        // The midi port will be selected at runtime by the user
        None,
        Some(midi_in_param),
    ) {
        println!("An error occured: {:?}", e);
    }
}
