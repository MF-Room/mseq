use mseq::{Conductor, DeteTrack, Instruction, MidiNote, Note, Track};

struct MyConductor {
    clk_div: DeteTrack,
}

impl Conductor for MyConductor {
    fn init(&mut self, context: &mut mseq::Context) {
        // The sequencer is on pause by default
        context.start();
        // Set the bpm to 157
        context.set_bpm(157);
    }

    fn update(&mut self, context: &mut mseq::Context) -> Vec<Instruction> {
        // Quit after 960 steps
        let step = context.get_step();

        // Quit after 960 steps
        if step == 959 {
            context.quit();
            return vec![];
        }

        // Play the clk_div track
        self.clk_div.play_step(step)
    }
}

fn main() {
    env_logger::init();

    let clk_div = mseq_tracks::div::load_from_file(
        "examples/res/clk_div_0.csv",
        MidiNote::new(Note::C, 4, 63),
        1,
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
