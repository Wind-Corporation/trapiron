#![feature(get_mut_unchecked)]

pub mod crash;
pub mod gui;

use gui::{Affine3, Mat4, OpaqueColor, Vec2, Vec3};

struct MyApplication {
    rect: gui::Primitive,
    animation_start: Option<std::time::Instant>,
}

const BLOCK_TEXTURES: gui::TextureGroup = gui::TextureGroup {};

impl MyApplication {
    fn new(gui: &mut gui::Gui) -> Self {
        println!("My init!");

        let texture = gui.texture(BLOCK_TEXTURES.id("test"));
        let white = OpaqueColor::rgb(Vec3::new(1.0, 1.0, 1.0));

        let rect_mesh = gui::Mesh::new(
            vec![
                gui::Vertex {
                    position: Vec3::new(-0.5, 0.5, 0.0),
                    color_multiplier: white,
                    texture_coords: Vec2::new(0.0, 1.0),
                },
                gui::Vertex {
                    position: Vec3::new(-0.5, -0.5, 0.0),
                    color_multiplier: white,
                    texture_coords: Vec2::new(0.0, 0.0),
                },
                gui::Vertex {
                    position: Vec3::new(0.5, 0.5, 0.0),
                    color_multiplier: white,
                    texture_coords: Vec2::new(1.0, 1.0),
                },
                gui::Vertex {
                    position: Vec3::new(0.5, -0.5, 0.0),
                    color_multiplier: white,
                    texture_coords: Vec2::new(1.0, 0.0),
                },
            ],
            vec![0, 1, 2, 3, 2, 1],
        )
        .expect("Could not make a rectangle");

        Self {
            rect: gui.make_primitive(rect_mesh, texture),
            animation_start: None,
        }
    }
}

fn draw_spinning(object: &mut impl gui::Drawable, t: f32, dcf: &mut gui::Dcf) {
    let dark = OpaqueColor::rgb(Vec3::new(0.1, 0.1, 0.15));

    let mut dcf = dcf.tfed(Affine3::from_rotation_y(t));
    let mut dcf = dcf.shifted(Vec3::X);

    object.draw(&mut dcf);
    object.draw(
        &mut dcf
            .tfed(Affine3::from_rotation_y(180f32.to_radians()))
            .colored(&dark),
    );
}

fn remap_depth(new_min: gui::Float, new_max: gui::Float) -> Mat4 {
    let mul = new_max - new_min;
    let add = new_min;
    Mat4::from_cols_array_2d(&[
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, mul, 0.0],
        [0.0, 0.0, add, 1.0],
    ])
}

impl gui::Application for MyApplication {}

impl gui::Drawable for MyApplication {
    fn draw(&mut self, dcf: &mut gui::Dcf) {
        // Draw 3D scene

        let mut new_settings = dcf.settings().clone();

        let fov = 75f32.to_radians();
        new_settings.screen_transform = remap_depth(0.1, 1.0) // takes up Z values 1.0 -> 0.1
            * Mat4::perspective_lh(fov, dcf.size().x / dcf.size().y, 0.01, 100.0);

        new_settings.view_transform = Affine3::look_at_lh(Vec3::Z * -2.5, Vec3::ZERO, Vec3::Y);

        dcf.set_settings(new_settings);

        let t = self.animation_start.get_or_insert(*dcf.time());
        let t = (*dcf.time() - *t).as_secs_f32();

        let blue = OpaqueColor::rgb(Vec3::new(0.0, 0.1, 0.9));
        let green = OpaqueColor::rgb(Vec3::new(0.05, 0.8, 0.1));

        draw_spinning(&mut self.rect, t * 1.0, &mut dcf.colored(&blue));
        draw_spinning(&mut self.rect, t * 1.5, dcf);
        draw_spinning(&mut self.rect, t * 0.8, &mut dcf.colored(&green));

        // Draw 2D overlay

        let mut new_settings = dcf.settings().clone();

        new_settings.screen_transform = remap_depth(0.0, 0.1) // takes up Z values 0.1 -> 0.0
            * Mat4::orthographic_lh(0.0, dcf.size().x, 0.0, dcf.size().y, 0.0, 1.0);
        new_settings.view_transform = Affine3::IDENTITY;

        dcf.set_settings(new_settings);

        self.rect.draw(
            &mut dcf
                .shifted(Vec3::new(48.0, 48.0, 0.0))
                .scaled(Vec3::splat(48.0 * 2.0)),
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
