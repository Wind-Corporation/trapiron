use std::time::Duration;

use crate::logic::Logic;

pub enum Event {
    Tick,
    PresentationTick { duration: Duration },
}

pub struct World;

impl World {
    pub fn new() -> Self {
        Self {}
    }

    pub fn process(&mut self, event: Event, _logic: &Logic) {
        match event {
            Event::Tick => println!("Tick!"),
            _ => (),
        }
    }

    pub fn process_presentation(&mut self, _event: &Event, _logic: &Logic) {
        println!("Presentation tick!");
    }
}
