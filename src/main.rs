#![feature(get_mut_unchecked)]

use glam::Vec3;

pub mod crash;
pub mod gui;

struct MyApplication {
    rect: gui::Primitive,
    animation_start: Option<std::time::Instant>,
}

const BLOCK_TEXTURES: gui::TextureGroup = gui::TextureGroup {};

impl MyApplication {
    fn new(gui: &mut gui::Gui) -> Self {
        println!("My init!");

        let texture = gui.texture(BLOCK_TEXTURES.id("test"));

        Self {
            rect: gui
                .make_primitive(
                    &[
                        gui::Vertex {
                            position: [-0.5, 0.5, 0.0],
                            color_multiplier: [1.0, 1.0, 1.0],
                            texture_coords: [0.0, 1.0],
                        },
                        gui::Vertex {
                            position: [-0.5, -0.5, 0.0],
                            color_multiplier: [1.0, 1.0, 1.0],
                            texture_coords: [0.0, 0.0],
                        },
                        gui::Vertex {
                            position: [0.5, 0.5, 0.0],
                            color_multiplier: [1.0, 1.0, 1.0],
                            texture_coords: [1.0, 1.0],
                        },
                        gui::Vertex {
                            position: [0.5, -0.5, 0.0],
                            color_multiplier: [1.0, 1.0, 1.0],
                            texture_coords: [1.0, 0.0],
                        },
                    ],
                    &[0, 1, 2, 3, 2, 1],
                    texture,
                )
                .expect("Could not make a rectangle"),
            animation_start: None,
        }
    }
}

fn draw_bouncy(object: &mut impl gui::Drawable, t: f32, dcf: &mut gui::Dcf) {
    // Move in screen space
    let mut dcf = dcf.shifted(Vec3::new((t * 1.0).sin() / 2.0, (t * 1.3).sin() / 2.0, 1.0));
    object.draw(&mut dcf.colored(&gui::OpaqueColor::rgb(Vec3::splat(0.1))));

    // Slow pulsing
    let mut dcf = dcf.scaled(Vec3::splat((t * 2.3).sin() * 0.3 + 0.7));
    object.draw(&mut dcf.colored(&gui::OpaqueColor::rgb(Vec3::splat(0.3))));

    // Fast wobble (demonstrates that move is not influenced by scale)
    let mut dcf = dcf.scaled(Vec3::new((t * 20.0).sin() * 0.1 + 0.9, 1.0, 1.0));
    object.draw(&mut dcf);
}

impl gui::Application for MyApplication {}

impl gui::Drawable for MyApplication {
    fn draw(&mut self, dcf: &mut gui::Dcf) {
        let t = self.animation_start.get_or_insert(*dcf.time());
        let t = (*dcf.time() - *t).as_secs_f32();

        let blue = gui::OpaqueColor::rgb(Vec3::new(0.0, 0.1, 0.9));
        let green = gui::OpaqueColor::rgb(Vec3::new(0.05, 0.8, 0.1));

        draw_bouncy(&mut self.rect, -t, &mut dcf.colored(&blue));
        draw_bouncy(&mut self.rect, t, dcf);
        draw_bouncy(
            &mut self.rect,
            t * 5.0,
            &mut dcf
                .shifted(Vec3::new(0.9, 0.9, 0.0))
                .scaled(Vec3::splat(0.1))
                .colored(&green),
        );
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
