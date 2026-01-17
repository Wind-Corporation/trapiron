//! Several aids and utilities for debugging graphics.

const DEBUG_TEXTURES: super::TextureGroup = super::TextureGroup {};

struct Axes(super::Primitive);

impl Axes {
    fn new(gui: &mut super::Gui) -> Self {
        let texture = gui.texture(&DEBUG_TEXTURES.id("axes"));
        let mesh = super::asset::load_mesh("axes");
        Self(gui.make_primitive(vec![mesh.bind(texture)]))
    }
}

struct Resources {
    axes: Axes,
}

impl Resources {
    fn new(gui: &mut super::Gui) -> Self {
        Self {
            axes: Axes::new(gui),
        }
    }
}

thread_local! {
    static RESOURCES: std::cell::RefCell<std::rc::Weak<Resources>> = const {
        std::cell::RefCell::new(std::rc::Weak::<Resources>::new())
    }
}

pub struct Initialization {
    _rc: std::rc::Rc<Resources>,
}

/// Initializes debug graphics assets and returns a shared ownership token.
///
/// Multiple initializations in parallel are safe: resources are freed when last `Rc` is dropped.
pub fn init(gui: &mut super::Gui) -> Initialization {
    RESOURCES.with_borrow_mut(|weak| {
        if let Some(rsrc) = weak.upgrade() {
            Initialization { _rc: rsrc }
        } else {
            let rsrc = std::rc::Rc::new(Resources::new(gui));
            *weak = std::rc::Rc::downgrade(&rsrc);
            Initialization { _rc: rsrc }
        }
    })
}

/// A graphics debugging aid; a primitive that displays the X, Y and Z axes.
///
/// The X axis is red, Y axis is green, Z axis is blue, similar to Blender UI.
///
/// Inside the primitive is a magenta box. It is normally hidden, but if the coordinate space is
/// inverted, it becomes visible.
pub fn axes(dcf: &mut super::Dcf) {
    RESOURCES.with_borrow(|weak_rsrc| {
        weak_rsrc
            .upgrade()
            .expect("Debug resources should be initialized with debug::init()")
            .axes
            .0
            .draw(dcf);
    })
}
