use crate::world::Event;

pub struct Control;

impl Control {
    pub fn new() -> Self {
        Self {}
    }

    pub fn fetch_into(&mut self, _out: &mut Vec<Event>) {
        // TODO
    }
}
