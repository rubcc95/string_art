pub trait Color {
    type Unit;

    fn r(&self) -> Self::Unit;
    fn g(&self) -> Self::Unit;
    fn b(&self) -> Self::Unit;

    #[inline]
    fn rgb(&self) -> [Self::Unit; 3] {
        [self.r(), self.g(), self.b()]
    }
}

impl<T: Copy> Color for (T, T, T) {
    type Unit = T;

    #[inline]
    fn r(&self) -> T {
        self.0
    }

    #[inline]
    fn g(&self) -> T {
        self.1
    }

    #[inline]
    fn b(&self) -> T {
        self.2
    }
}

impl<T: Copy> Color for [T; 3] {
    type Unit = T;

    #[inline]
    fn r(&self) -> T {
        self[0]
    }

    #[inline]
    fn g(&self) -> T {
        self[1]
    }

    #[inline]
    fn b(&self) -> T {
        self[2]
    }
}

#[cfg(feature = "image")]
impl<T: Copy> Color for image::Rgb<T> {
    type Unit = T;

    fn r(&self) -> T {
        self.0[0]
    }

    fn g(&self) -> T {
        self.0[0]
    }

    fn b(&self) -> T {
        self.0[0]
    }
}
