pub mod client;
pub mod crash;
pub mod gui;
pub mod logic;
pub mod world;

struct MyApplication {
    game: client::Game,
}

impl MyApplication {
    fn new(gui: &mut gui::Gui) -> Self {
        println!("My init!");

        Self {
            game: client::Game::new(gui),
        }
    }
}

impl gui::Application for MyApplication {}

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
