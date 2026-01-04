use crate::world::Event;

pub struct Control {
    count: u32,
}

impl Control {
    pub fn new() -> Self {
        Self { count: 0 }
    }

    pub fn fetch_into(&mut self, out: &mut Vec<Event>) {
        self.count += 1;
        if self.count > 64 {
            self.count = 0;
            println!("Boop!");
            out.push(Event::MoveCamera);
        }
    }
}
