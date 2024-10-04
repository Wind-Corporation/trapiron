#![feature(get_mut_unchecked)]

mod crash;
mod gui;

struct MyApplication {
    triangle: gui::Primitive3,
}

const BLOCK_TEXTURES: gui::TextureGroup = gui::TextureGroup {};

impl MyApplication {
    fn new(gui: &mut gui::Gui) -> Self {
        println!("My init!");

        let texture = gui.texture(BLOCK_TEXTURES.id("test"));

        Self {
            triangle: gui
                .make_primitive3(
                    &[
                        gui::Vertex3 {
                            position: [-0.5, 0.5, 0.0],
                            color_multiplier: [1.0, 1.0, 1.0],
                            texture_coords: [0.0, 1.0],
                        },
                        gui::Vertex3 {
                            position: [-0.5, -0.5, 0.0],
                            color_multiplier: [1.0, 1.0, 1.0],
                            texture_coords: [0.0, 0.0],
                        },
                        gui::Vertex3 {
                            position: [0.5, 0.5, 0.0],
                            color_multiplier: [1.0, 1.0, 1.0],
                            texture_coords: [1.0, 1.0],
                        },
                        gui::Vertex3 {
                            position: [0.5, -0.5, 0.0],
                            color_multiplier: [1.0, 1.0, 1.0],
                            texture_coords: [1.0, 0.0],
                        },
                    ],
                    &[0, 1, 2, 3, 2, 1],
                    texture,
                )
                .expect("Could not make a triangle"),
        }
    }
}

impl gui::Application for MyApplication {}

impl gui::Drawable for MyApplication {
    fn draw(&mut self, ctxt: &mut gui::DrawContext) {
        self.triangle.draw(ctxt);
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
