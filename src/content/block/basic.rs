//! Block kinds without complicated code.

use crate::content::block::*;

pub struct AirKind;

impl KindInstance for AirKind {
    fn new(_: &mut Gui) -> Self {
        Self
    }
}

pub struct AirView;

impl ViewInstance for AirView {}
impl Drawable for AirView {
    fn draw(&mut self, _: &mut crate::gui::Dcf) {
        // Do nothing
    }
}

pub struct Air;

impl Instance for Air {
    type Kind = AirKind;
    type View = AirView;
    fn view(&self, _: &Self::Kind) -> Self::View {
        AirView
    }
    fn from(_: &Serialized) -> Self {
        Self {}
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct PusherKind {
    model: PusherView,
}

impl KindInstance for PusherKind {
    fn new(gui: &mut Gui) -> Self {
        let texture = gui.texture(&TEXTURES.id("pusher"));
        let mesh = crate::gui::asset::load_mesh("pusher");
        let primitive = Rc::new(gui.make_primitive(vec![mesh.bind(texture)]));
        Self {
            model: PusherView(primitive),
        }
    }
}

#[derive(Clone)]
pub struct PusherView(Rc<Primitive>);

impl ViewInstance for PusherView {}
impl Drawable for PusherView {
    fn draw(&mut self, dcf: &mut crate::gui::Dcf) {
        self.0.draw(dcf);
    }
}

pub struct Pusher;

impl Instance for Pusher {
    type Kind = PusherKind;
    type View = PusherView;
    fn view(&self, rsrc: &Self::Kind) -> Self::View {
        rsrc.model.clone()
    }
    fn from(_: &Serialized) -> Self {
        Self {}
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct StoneKind {
    model: FullCube,
}

impl KindInstance for StoneKind {
    fn new(gui: &mut Gui) -> Self {
        Self {
            model: FullCube::new(&gui.texture(&TEXTURES.id("stone")), gui),
        }
    }
}

pub struct Stone;

impl Instance for Stone {
    type Kind = StoneKind;
    type View = FullCube;
    fn view(&self, rsrc: &Self::Kind) -> Self::View {
        rsrc.model.clone()
    }
    fn from(_: &Serialized) -> Self {
        Self {}
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct SandKind {
    model: FullCube,
}

impl KindInstance for SandKind {
    fn new(gui: &mut Gui) -> Self {
        Self {
            model: FullCube::new(&gui.texture(&TEXTURES.id("sand")), gui),
        }
    }
}

pub struct Sand;

impl Instance for Sand {
    type Kind = SandKind;
    type View = FullCube;
    fn view(&self, rsrc: &Self::Kind) -> Self::View {
        rsrc.model.clone()
    }
    fn from(_: &Serialized) -> Self {
        Self {}
    }
}
