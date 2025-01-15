use mseq::{Conductor, DeteTrack, MidiNote, Note};

struct MyConductor {
    clk_div: DeteTrack,
}

impl Conductor for MyConductor {
    fn init(&mut self, context: &mut mseq::Context<impl mseq::MidiOut>) {
        // The sequencer is on pause by default
        context.start();
        // Set the bpm to 157
        context.set_bpm(157);
    }

    fn update(&mut self, context: &mut mseq::Context<impl mseq::MidiOut>) {
        // Play the clk_div track
        context.midi.play_track(&mut self.clk_div);

        // Quit after 960 steps
        if context.get_step() == 959 {
            context.quit();
        }
    }
}

fn main() {
    env_logger::init();

    let clk_div = DeteTrack::load_clock_div_from_file(
        "examples/res/clk_div_0.csv",
        MidiNote::new(Note::C, 4, 63),
        0,
        "my_clk_div",
    )
    .unwrap();

    if let Err(e) = mseq::run(
        MyConductor { clk_div },
        // The midi port will be selected at runtime by the user
        None,
    ) {
        println!("An error occured: {:?}", e);
    }
}
