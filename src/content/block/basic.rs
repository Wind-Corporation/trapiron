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
    fn view(&self, _: &Self::Kind, _: &Resources) -> Self::View {
        AirView
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
    fn view(&self, rsrc: &Self::Kind, _: &Resources) -> Self::View {
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
    fn view(&self, rsrc: &Self::Kind, _: &Resources) -> Self::View {
        rsrc.model.clone()
    }
    fn from(_: &Serialized) -> Self {
        Self {}
    }
}
