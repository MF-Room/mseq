use std::sync::{Arc, Mutex};

use crate::Conductor;

pub trait InputManager<T: Conductor> {
    /// Can be blocking
    fn acquire_inputs(&mut self);

    /// Warning: Cannot be blocking
    fn handle_inputs(&mut self, conductor: &mut T);
}

pub fn input_loop<T: Conductor>(
    mut input_manager: impl InputManager<T>,
    conductor: &Arc<Mutex<T>>,
) {
    loop {
        input_manager.acquire_inputs();
        let c = &mut conductor.lock().unwrap();
        input_manager.handle_inputs(c);
    }
}
