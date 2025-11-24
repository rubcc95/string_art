pub trait Color32: Color<Unit = u8> {}

impl<T: Color<Unit = u8>> Color32 for T {}

pub trait Color {
    type Unit;

    fn r(&self) -> Self::Unit;
    fn g(&self) -> Self::Unit;
    fn b(&self) -> Self::Unit;

    fn rgb(&self) -> [Self::Unit; 3] {
        [self.r(), self.g(), self.b()]
    }
}

impl<T: Copy> Color for (T, T, T) {
    type Unit = T;

    fn r(&self) -> T {
        self.0
    }

    fn g(&self) -> T {
        self.1
    }

    fn b(&self) -> T {
        self.2
    }
}

impl<T: Copy> Color for [T; 3] {
    type Unit = T;

    fn r(&self) -> T {
        let [r, _, _] = *self;
        r
    }

    fn g(&self) -> T {
        let [_, g, _] = *self;
        g
    }

    fn b(&self) -> T {
        let [_, _, b] = *self;
        b
    }
}

#[cfg(feature = "image")]
mod from_image {
    use super::*;
    impl<T: Copy> Color for image::Rgb<T> {
        type Unit = T;

        fn r(&self) -> T {
            let [r, _, _] = self.0;
            r
        }

        fn g(&self) -> T {
            let [_, g, _] = self.0;
            g
        }

        fn b(&self) -> T {
            let [_, _, b] = self.0;
            b
        }
    }
}
