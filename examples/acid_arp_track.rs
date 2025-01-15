use mseq::{Conductor, DeteTrack};

struct MyConductor {
    acid: DeteTrack,
    arp: DeteTrack,
}

impl Conductor for MyConductor {
    fn init(&mut self, context: &mut mseq::Context<impl mseq::MidiOut>) {
        // The sequencer is on pause by default
        context.start();
    }

    fn update(&mut self, context: &mut mseq::Context<impl mseq::MidiOut>) {
        // First play acid on channel 0 and then arp on channel 1
        if context.get_step() < 430 {
            context.midi.play_track(&mut self.acid);
        } else {
            context.midi.play_track(&mut self.arp);
        }

        // Quit after 960 steps
        if context.get_step() == 959 {
            context.quit();
        }
    }
}

fn main() {
    env_logger::init();

    let acid =
        DeteTrack::load_acid_from_file("examples/res/acid_0.csv", mseq::Note::A, 0, "my_acid")
            .unwrap();
    let arp = DeteTrack::load_arp_from_file(
        "examples/res/arp_0.csv",
        mseq::ArpDiv::T8,
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
