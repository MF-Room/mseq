use mseq::{Conductor, DeteTrack};

struct MyConductor {
    track: DeteTrack,
}

impl Conductor for MyConductor {
    fn init(&mut self, context: &mut mseq::Context<impl mseq::MidiOut>) {
        // The sequencer is on pause by default
        context.start();
    }

    fn update(&mut self, context: &mut mseq::Context<impl mseq::MidiOut>) {
        // The conductor plays the track
        context.midi.play_track(&mut self.track);

        // Quit after 960 steps
        if context.get_step() == 959 {
            context.quit();
        }
    }
}

fn main() {
    env_logger::init();

    let track =
        mseq::track::load_from_file("examples/res/track_0.mid", mseq::Note::A, 1, "my_track")
            .unwrap();

    if let Err(e) = mseq::run(
        MyConductor { track },
        // The midi port will be selected at runtime by the user
        None,
    ) {
        println!("An error occured: {:?}", e);
    }
}
