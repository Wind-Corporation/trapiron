//! Block kinds without complicated code.

use crate::content::block::*;

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
