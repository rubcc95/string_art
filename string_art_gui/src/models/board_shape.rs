use super::*;

    #[derive(Clone, Copy, PartialEq, Default)]
    pub struct BoardShape {
        pub mode: Mode,
        pub ellipse: input::board_shape::Ellipse,
        pub rectangle: Rectangle,
    }

    impl BoardShape {
        pub fn set(&mut self, input: input::BoardShape) {
            match input {
                input::BoardShape::Ellipse(ellipse) => {
                    self.mode = Mode::Ellipse;
                    self.ellipse = ellipse;
                }
                input::BoardShape::Rectangle(rectangle) => {
                    self.mode = Mode::Rectangle;
                    self.rectangle.set(rectangle);
                }
            }
        }
    }

    impl Into<input::BoardShape> for BoardShape {
        fn into(self) -> input::BoardShape {
            match self.mode {
                Mode::Ellipse => input::BoardShape::Ellipse(self.ellipse),
                Mode::Rectangle => input::BoardShape::Rectangle(self.rectangle.into()),
            }
        }
    }

    #[derive(Clone, Copy, PartialEq, Default)]
    pub enum Mode {
        #[default]
        Ellipse,
        Rectangle,
    }

    pub use rectangle::Rectangle;

    pub mod rectangle {
        use super::*;

        #[derive(Props, Clone, Copy, PartialEq, Default)]
        pub struct Rectangle {
            pub mode: Mode,
            pub auto: input::board_shape::rectangle::Auto,
            pub manual: input::board_shape::rectangle::Manual,
        }

        impl Rectangle{
            pub fn set(&mut self, input: input::board_shape::Rectangle){
                match input {
                    input::board_shape::Rectangle::Auto(auto) => {
                        self.mode = Mode::Auto;
                        self.auto = auto;
                    }
                    input::board_shape::Rectangle::Manual(manual) => {
                        self.mode = Mode::Manual;
                        self.manual = manual;
                    }
                }
            }
        }
        
        impl Into<input::board_shape::Rectangle> for Rectangle{
            fn into(self) -> input::board_shape::Rectangle {
                match self.mode{
                    Mode::Auto => input::board_shape::Rectangle::Auto(self.auto),
                    Mode::Manual => input::board_shape::Rectangle::Manual(self.manual),
                }
            }
        }

        #[derive(Clone, Copy, PartialEq, Default)]
        pub enum Mode {
            #[default]
            Auto,
            Manual,
        }
    }
