use crate::{Context, MidiConnection};

pub trait Conductor {
    fn init(&mut self, context: &mut Context<impl MidiConnection>);
    fn update(&mut self, context: &mut Context<impl MidiConnection>);
}
