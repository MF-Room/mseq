use mseq::Conductor;
use mseq::MidiNote;
use mseq::Track;

struct ExampleTrack {}

// Implement a track for full freedom (randomization, automatization...)
impl Track for ExampleTrack {
    fn play_step(
        &mut self,
        step: u32,
        midi_controller: &mut mseq::MidiController<impl mseq::MidiConnection>,
    ) {
        // Midi channel id to send the note to
        let channel_id = 8;
        if step % 8 == 0 {
            // Describe the midi note to send
            let note = MidiNote {
                note: mseq::Note::B,
                octave: 3,
                vel: 127,
            };
            // Note length in number of steps
            let note_length = 3;

            // Request to play the note to the midi controller.
            midi_controller.play_note(note, note_length, channel_id);
        }
    }
}

struct ExampleConductor {
    track: ExampleTrack,
}

impl Conductor for ExampleConductor {
    fn init(&mut self, context: &mut mseq::Context<impl mseq::MidiConnection>) {
        // The sequencer is on pause by default
        context.start();
    }

    fn update(&mut self, context: &mut mseq::Context<impl mseq::MidiConnection>) {
        // The conductor can play tracks...
        context.midi.play_track(&mut self.track);

        // ...or play notes directly
        if context.get_step() % 24 == 0 {
            context.midi.play_note(
                MidiNote {
                    note: mseq::Note::D,
                    octave: 3,
                    vel: 127,
                },
                12,
                1,
            );
        }
    }
}

fn main() {
    if let Err(e) = mseq::run(
        ExampleConductor {
            track: ExampleTrack {},
        },
        // The midi port will be selected at runtime by the user
        None,
    ) {
        println!("An error occured: {:?}", e);
    }
}
