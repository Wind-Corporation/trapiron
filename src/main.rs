use std::rc::Rc;

pub mod client;
pub mod content;
pub mod crash;
pub mod gui;
pub mod logic;
pub mod world;

struct MyApplication {
    game: client::Game,
}

impl MyApplication {
    fn new(gui: &mut gui::Gui) -> Self {
        println!("Loading resources");
        let resources = Rc::new(content::Resources::new(gui));

        println!("Starting game");
        Self {
            game: client::Game::new(resources, gui),
        }
    }
}

impl gui::Application for MyApplication {
    fn on_input(&mut self, input: gui::Input, gui: &mut gui::Gui) {
        self.game.on_input(input, gui);
    }
}

impl gui::Drawable for MyApplication {
    fn draw(&mut self, dcf: &mut gui::Dcf) {
        self.game.tick(*dcf.time());
        self.game.draw(dcf);
    }
}

fn main() {
    crash::setup_panic_hook();

    crash::with_context(("Thread", || "main"), || {
        println!("My early init!");
        gui::backend::run(MyApplication::new);
        println!("My late shutdown!");
    });
}
