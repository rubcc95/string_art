#[derive(Clone, Copy, Debug, Default)]
pub struct Anchor<L> {
    pub idx: usize,
    pub link: L,
}

impl<L: std::fmt::Display> std::fmt::Display for Anchor<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.idx, self.link)
    }
}
