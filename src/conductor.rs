use crate::{Context, MidiConnection};

pub trait Conductor<T: MidiConnection> {
    fn init(&mut self, context: &mut Context<T>);
    fn update(&mut self, context: &mut Context<T>);
}
