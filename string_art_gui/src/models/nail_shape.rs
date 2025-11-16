use super::*;

#[derive(Clone, Copy, PartialEq)]
pub struct NailShape {
    pub mode: Mode,
    pub circular: f32,
    pub hook: f32,
}

impl NailShape {
    pub fn set(&mut self, input: input::NailShape) {
        match input {
            input::NailShape::Circular(circular) => {
                self.mode = Mode::Circular;
                self.circular = circular;
            }
            input::NailShape::Hook(hook) => {
                self.mode = Mode::Hook;
                self.hook = hook;
            }
            input::NailShape::Point => {
                self.mode = Mode::Point;
            }
        }
    }
}

impl Into<input::NailShape> for NailShape {
    fn into(self) -> input::NailShape {
        match self.mode {
            Mode::Circular => input::NailShape::Circular(self.circular),
            Mode::Hook => input::NailShape::Hook(self.hook),
            Mode::Point => input::NailShape::Point,
        }
    }
}

impl Default for NailShape {
    fn default() -> Self {
        Self {
            mode: Default::default(),
            circular: 0.234375,
            hook: Default::default(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Default)]
pub enum Mode {
    #[default]
    Circular,
    Hook,
    Point,
}
