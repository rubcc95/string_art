#[derive(Clone, Copy, Debug, Default)]
pub struct Anchor<L> {
    pub idx: usize,
    pub link: L,
}

impl <L: std::fmt::Display> std::fmt::Display for Anchor<L>{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.idx, self.link)
    }
}

pub struct Debugger<'a, H, L> {
    pub handle: &'a H,
    pub anchor: &'a Anchor<L>,
}

impl<B: super::Handle> core::fmt::Debug for Debugger<'_, B, B::Link> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Anchor")
            .field("idx", &self.anchor.idx)
            .field("link", &self.handle.index_of(&self.anchor.link))
            .finish()
    }
}