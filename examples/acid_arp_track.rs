use mseq::{Conductor, DeteTrack, Instruction, Track};

struct MyConductor {
    acid: DeteTrack,
    arp: DeteTrack,
}

impl Conductor for MyConductor {
    fn init(&mut self, context: &mut mseq::Context) {
        // The sequencer is on pause by default
        context.start();
    }

    fn update(&mut self, context: &mut mseq::Context) -> Vec<Instruction> {
        let step = context.get_step();
        // Quit after 960 steps
        if context.get_step() == 959 {
            context.quit();
        }

        // First play acid on channel 0 and then arp on channel 1
        if step < 430 {
            self.acid.play_step(step)
        } else {
            self.arp.play_step(step)
        }
    }
}

fn main() {
    env_logger::init();

    let acid =
        mseq_tracks::acid::load_from_file("examples/res/acid_0.csv", mseq::Note::A, 1, "my_acid")
            .unwrap();
    let arp = mseq_tracks::arp::load_from_file(
        "examples/res/arp_0.csv",
        mseq_tracks::arp::ArpDiv::T8,
        mseq::Note::A,
        1,
        "my_arp",
    )
    .unwrap();

    if let Err(e) = mseq::run(
        MyConductor { acid, arp },
        // The midi port will be selected at runtime by the user
        None,
    ) {
        println!("An error occured: {:?}", e);
    }
}
