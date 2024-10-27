#![feature(get_mut_unchecked)]

use glam::{Affine3A, Mat4, Vec3};

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

fn draw_spinning(object: &mut impl gui::Drawable, t: f32, dcf: &mut gui::Dcf) {
    let dark = gui::OpaqueColor::rgb(Vec3::new(0.1, 0.1, 0.15));

    let mut dcf = dcf.tfed(Affine3A::from_rotation_y(t));
    let mut dcf = dcf.shifted(Vec3::X);

    object.draw(&mut dcf);
    object.draw(
        &mut dcf
            .tfed(Affine3A::from_rotation_y(180f32.to_radians()))
            .colored(&dark),
    );
}

impl gui::Application for MyApplication {}

impl gui::Drawable for MyApplication {
    fn draw(&mut self, dcf: &mut gui::Dcf) {
        let mut new_settings = dcf.settings().clone();

        let fov = 75f32.to_radians();
        new_settings.screen_transform =
            Mat4::perspective_lh(fov, dcf.size().x / dcf.size().y, 0.01, 100.0);

        new_settings.view_transform = Affine3A::look_at_lh(Vec3::Z * -2.5, Vec3::ZERO, Vec3::Y);

        dcf.set_settings(new_settings);

        let t = self.animation_start.get_or_insert(*dcf.time());
        let t = (*dcf.time() - *t).as_secs_f32();

        let blue = gui::OpaqueColor::rgb(Vec3::new(0.0, 0.1, 0.9));
        let green = gui::OpaqueColor::rgb(Vec3::new(0.05, 0.8, 0.1));

        draw_spinning(&mut self.rect, t * 1.0, &mut dcf.colored(&blue));
        draw_spinning(&mut self.rect, t * 1.5, dcf);
        draw_spinning(&mut self.rect, t * 0.8, &mut dcf.colored(&green));
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
