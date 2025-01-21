pub enum Message {
    CreatingNail(usize),
    Baking,
    Dithering(usize, usize),
    Computing(usize),
}

pub trait Verboser {
    fn verbose(&mut self, message: Message);
}

pub struct Silent;

impl Verboser for Silent {
    fn verbose(&mut self, _: Message) {}
}
