use mseq::{Conductor, MidiNote, Track};
use rand::{thread_rng, Rng};

struct MyTrack {
    channel_id: u8,
}

// Implement a track for full freedom (randomization, automatization...)
impl Track for MyTrack {
    fn play_step(
        &mut self,
        step: u32,
        midi_controller: &mut mseq::MidiController<impl mseq::MidiConnection>,
    ) {
        // Midi channel id to send the note to
        if step % 8 == 0 {
            // Choose a random note
            let note = MidiNote {
                note: thread_rng().gen::<u8>().into(),
                octave: 4,
                vel: 127,
            };

            // Note length in number of steps
            let note_length = 3;

            // Request to play the note to the midi controller.
            midi_controller.play_note(note, note_length, self.channel_id);
        }
    }
}

struct MyConductor {
    track: MyTrack,
}

impl Conductor for MyConductor {
    fn init(&mut self, context: &mut mseq::Context<impl mseq::MidiConnection>) {
        // The sequencer is on pause by default
        context.start();
    }

    fn update(&mut self, context: &mut mseq::Context<impl mseq::MidiConnection>) {
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

    if let Err(e) = mseq::run(
        MyConductor {
            track: MyTrack { channel_id: 0 },
        },
        // The midi port will be selected at runtime by the user
        None,
    ) {
        println!("An error occured: {:?}", e);
    }
}
