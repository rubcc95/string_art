use super::*;

#[derive(Clone, Copy, PartialEq)]
pub struct PipelineSettings {
    pub mode: Mode,
    pub mono: u32,
}

impl PipelineSettings {
    pub fn set(&mut self, input: input::pipeline::Mode) {
        match input {
            input::pipeline::Mode::Mono(val) => {
                self.mode = Mode::Mono;
                self.mono = val;
            } 
            input::pipeline::Mode::Multi => self.mode = Mode::Multi,
        }
    }
}

impl Into<input::pipeline::Mode> for PipelineSettings {
    fn into(self) -> input::pipeline::Mode {
        match self.mode {
            Mode::Mono => input::pipeline::Mode::Mono(self.mono),
            Mode::Multi => input::pipeline::Mode::Multi,
        }
    }
}

impl Default for PipelineSettings {
    fn default() -> Self {
        Self {
            mode: Default::default(),
            mono: 4000,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Default)]
pub enum Mode {
    #[default]
    Mono,
    Multi,
}
