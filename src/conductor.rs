use crate::Context;

pub trait Conductor {
    fn init(&mut self, context: &mut Context);
    fn update(&mut self, context: &mut Context);
}
