#![feature(get_mut_unchecked)]

use crate::gui::Drawable3;
use glam::{Affine3A, Vec3};

mod crash;
mod gui;

struct MyApplication {
    triangle: gui::Primitive3,
    animation_start: Option<std::time::Instant>,
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
            animation_start: None,
        }
    }
}

impl gui::Application for MyApplication {
    fn draw(&mut self, ctxt: &mut gui::DrawContext) {
        let mut dcf = ctxt.start_3();

        let t = self
            .animation_start
            .get_or_insert_with(|| std::time::Instant::now());
        let t = (*dcf.time() - *t).as_secs_f32();

        let transform = Affine3A::from_scale_rotation_translation(
            Vec3::splat((t * 2.3).sin() * 0.3 + 0.7),
            Default::default(),
            Vec3::new((t * 1.0).sin() / 2.0, (t * 1.3).sin() / 2.0, 1.0),
        );

        self.triangle.draw(&mut dcf.tfed(transform));
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
