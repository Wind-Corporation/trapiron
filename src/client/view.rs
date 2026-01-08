//! Graphical presentation of [`World`].

use crate::{
    gui::{Affine3, Drawable, Float, Mat4, OpaqueColor, Vec3},
    world::World,
};

/// Renderer of [`World`], including 3D model and HUD controlled by simulation.
pub struct View {
    rect: crate::gui::Primitive,
    cube: crate::gui::Primitive,
    axes: crate::gui::Primitive,
    animation_start: Option<std::time::Instant>,
}

/// Possible configurations for camera anchor and view angle.
#[derive(Debug, Clone)]
pub enum Camera {
    /// A detached camera controlled entirely by presentation.
    Free {
        /// Absolute position in world.
        position: crate::world::Vec3,
        /// Rotation from world coordinate frame to camera frame of reference.
        rotation: crate::gui::Quat,
    },
}

impl Camera {
    /// Determine position and rotation of the camera in world coordinate frame.
    ///
    /// Rotation is specified from world coordinate frame to camera frame of reference.
    fn resolve(&self, _world: &World) -> (crate::gui::Vec3, crate::gui::Quat) {
        match self {
            Camera::Free { position, rotation } => (*position, *rotation),
        }
    }
}

/// Dynamically configurable settings for rendering a single frame.
#[derive(Debug, Clone)]
pub struct Parameters {
    /// Camera anchor and view angle.
    pub camera: Camera,
    /// Horizontal field of view in radians. Vertical field of view is determined based on frame
    /// aspect ratio.
    pub fov: crate::gui::Float,
}

const BLOCK_TEXTURES: crate::gui::TextureGroup = crate::gui::TextureGroup {};

impl View {
    pub fn new(gui: &mut crate::gui::Gui) -> Self {
        let texture = gui.texture(BLOCK_TEXTURES.id("test"));
        let rect = crate::gui::Mesh::square(1.0)
            .centered()
            .bind(texture.clone());
        let cube =
            crate::gui::Mesh::tmp_ppp(Vec3::splat(-0.5), Vec3::X, Vec3::Y, Vec3::Z, &texture);

        Self {
            rect: gui.make_primitive(vec![rect]),
            cube: gui.make_primitive(cube),
            axes: crate::gui::debug::axes(gui),
            animation_start: None,
        }
    }
}

fn draw_spinning(object: &mut impl crate::gui::Drawable, t: f32, dcf: &mut crate::gui::Dcf) {
    let dark = OpaqueColor::rgb(Vec3::new(0.1, 0.1, 0.15));

    let mut dcf = dcf.tfed(Affine3::from_rotation_z(t));
    let mut dcf = dcf.shifted(Vec3::X * 2.0);

    object.draw(&mut dcf.tfed(Affine3::from_rotation_x(90f32.to_radians())));
    object.draw(
        &mut dcf
            .tfed(Affine3::from_rotation_z(180f32.to_radians()))
            .tfed(Affine3::from_rotation_x(90f32.to_radians()))
            .colored(&dark),
    );
}

fn remap_depth(new_min: Float, new_max: crate::gui::Float) -> Mat4 {
    let mul = new_max - new_min;
    let add = new_min;
    Mat4::from_cols_array_2d(&[
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, mul, 0.0],
        [0.0, 0.0, add, 1.0],
    ])
}

impl View {
    pub fn draw(&mut self, dcf: &mut crate::gui::Dcf, world: &World, params: &Parameters) {
        // Draw 3D scene

        let t = self.animation_start.get_or_insert(*dcf.time());
        let t = (*dcf.time() - *t).as_secs_f32();

        let mut new_settings = dcf.settings().clone();

        new_settings.screen_transform = remap_depth(0.1, 1.0) // takes up Z values 1.0 -> 0.1
            * Mat4::perspective_rh(params.fov, dcf.size().x / dcf.size().y, 0.01, 100.0);

        let (camera_pos, camera_rot) = params.camera.resolve(world);
        new_settings.view_transform = Affine3::look_at_rh(Vec3::ZERO, Vec3::X, Vec3::Z)
            * Affine3::from_quat(-camera_rot)
            * Affine3::from_translation(-camera_pos);

        new_settings.lighting = crate::gui::draw::Lighting {
            ambient_color: OpaqueColor::rgb(Vec3::new(0.1, 0.15, 0.3)),
            diffuse_color: OpaqueColor::rgb(Vec3::new(0.9, 0.85, 0.6)),
            diffuse_direction: Vec3::new(1.0, 2.0, -3.0).normalize(),
        };

        dcf.set_settings(new_settings);

        self.axes.draw(dcf);

        let blue = OpaqueColor::rgb(Vec3::new(0.0, 0.1, 0.9));
        let green = OpaqueColor::rgb(Vec3::new(0.05, 0.8, 0.1));

        draw_spinning(&mut self.rect, t * 1.0, &mut dcf.colored(&blue));
        draw_spinning(&mut self.rect, t * 1.5, dcf);
        draw_spinning(&mut self.rect, t * 0.8, &mut dcf.colored(&green));

        self.rect
            .draw(&mut dcf.shifted(Vec3::Z * -3.0).scaled(Vec3::splat(10.0)));

        self.cube.draw(
            &mut dcf
                .tfed(Affine3::from_rotation_z(t))
                .tfed(Affine3::from_rotation_x(t * 1.3))
                .tfed(Affine3::from_rotation_y(t * 0.7)),
        );

        // Draw 2D overlay

        let mut new_settings = dcf.settings().clone();

        new_settings.screen_transform = remap_depth(0.0, 0.1) // takes up Z values 0.1 -> 0.0
            * Mat4::orthographic_rh(0.0, dcf.size().x, 0.0, dcf.size().y, 0.0, 1.0);
        new_settings.view_transform = Affine3::IDENTITY;
        new_settings.lighting = Default::default();

        dcf.set_settings(new_settings);

        let tint = if dcf.gui().cursor_captured() {
            &OpaqueColor::WHITE
        } else {
            &blue
        };
        self.rect.draw(
            &mut dcf
                .shifted(Vec3::new(48.0, 48.0, 0.0))
                .scaled(Vec3::splat(48.0 * 2.0))
                .colored(tint),
        );
    }
}
