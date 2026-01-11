use crate::content::block::*;

pub struct PusherKind {
    model_compressed: Rc<Primitive>,
    model_extended: Rc<Primitive>,
}

impl KindInstance for PusherKind {
    fn new(gui: &mut Gui) -> Self {
        let texture = gui.texture(&TEXTURES.id("pusher"));

        let mut model = |name: &str| {
            let mesh = crate::gui::asset::load_mesh(name);
            Rc::new(gui.make_primitive(vec![mesh.bind(texture.clone())]))
        };

        Self {
            model_compressed: model("pusher_compressed"),
            model_extended: model("pusher_extended"),
        }
    }
}

pub struct PusherView {
    pusher: Rc<Primitive>,
    contents: Box<View>,
}

impl ViewInstance for PusherView {}
impl Drawable for PusherView {
    fn draw(&mut self, dcf: &mut crate::gui::Dcf) {
        self.pusher.draw(dcf);
        self.contents
            .draw(&mut dcf.scaled(crate::gui::Vec3::splat(0.5)));
    }
}

pub enum Pusher {
    Holds(Box<Block>),
    Extended,
}

impl Instance for Pusher {
    type Kind = PusherKind;
    type View = PusherView;

    fn view(&self, kind: &Self::Kind, rsrc: &Resources) -> Self::View {
        match self {
            Self::Holds(contents) => Self::View {
                pusher: kind.model_compressed.clone(),
                contents: Box::new(contents.view(rsrc)),
            },
            Self::Extended => Self::View {
                pusher: kind.model_extended.clone(),
                contents: Box::new(View::Air(AirView)),
            },
        }
    }

    fn from(data: &Serialized) -> Self {
        match data.0 {
            0 => Self::Holds(Box::new(Block::Air(Air))),
            1 => Self::Holds(Box::new(Block::Sand(Sand))),
            _ => Self::Extended,
        }
    }
}
