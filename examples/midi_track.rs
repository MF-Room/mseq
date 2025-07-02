use mseq::{Conductor, DeteTrack, Instruction, Track};

struct MyConductor {
    track: DeteTrack,
}

impl Conductor for MyConductor {
    fn init(&mut self, context: &mut mseq::Context) {
        // The sequencer is on pause by default
        context.start();
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

    let track =
        mseq_tracks::midi::load_from_file("examples/res/track_0.mid", mseq::Note::A, 1, "my_track")
            .unwrap();

    if let Err(e) = mseq::run(
        MyConductor { track },
        // The midi port will be selected at runtime by the user
        None,
        None,
    ) {
        println!("An error occured: {:?}", e);
    }
}
