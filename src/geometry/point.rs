use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use num_traits::AsPrimitive;

    use crate::Float;

    #[derive(Copy, Clone, Debug)]
    pub struct Point<T> {
        pub x: T,
        pub y: T,
    }

    impl<T: Add<S>, S> Add<Point<S>> for Point<T>{
        type Output = Point<T::Output>;
    
        fn add(self, rhs: Point<S>) -> Self::Output {
            Point{
                x: self.x + rhs.x,
                y: self.y + rhs.y
            }
        }
    }

    impl<T: Sub<S>, S> Sub<Point<S>> for Point<T>{
        type Output = Point<T::Output>;
    
        fn sub(self, rhs: Point<S>) -> Self::Output {
            Point{
                x: self.x - rhs.x,
                y: self.y - rhs.y
            }
        }
    }

    impl<T: Mul<S>, S> Mul<Point<S>> for Point<T>{
        type Output = Point<T::Output>;
    
        fn mul(self, rhs: Point<S>) -> Self::Output {
            Point{
                x: self.x * rhs.x,
                y: self.y * rhs.y
            }
        }
    }

    impl<T: Div<S>, S> Div<Point<S>> for Point<T>{
        type Output = Point<T::Output>;
    
        fn div(self, rhs: Point<S>) -> Self::Output {
            Point{
                x: self.x / rhs.x,
                y: self.y / rhs.y
            }
        }
    }

    impl<T: AddAssign<S>, S> AddAssign<Point<S>> for Point<T>{
        fn add_assign(&mut self, rhs: Point<S>) {
            self.x += rhs.x;
            self.y += rhs.y;
        }
    }

    impl<T: SubAssign<S>, S> SubAssign<Point<S>> for Point<T>{
        fn sub_assign(&mut self, rhs: Point<S>) {
            self.x -= rhs.x;
            self.y -= rhs.y;
        }
    }

    impl<T: MulAssign<S>, S> MulAssign<Point<S>> for Point<T>{
        fn mul_assign(&mut self, rhs: Point<S>) {
            self.x *= rhs.x;
            self.y *= rhs.y;
        }
    }

    impl<T: DivAssign<S>, S> DivAssign<Point<S>> for Point<T>{
        fn div_assign(&mut self, rhs: Point<S>) {
            self.x /= rhs.x;
            self.y /= rhs.y;
        }
    }

    impl<T: Neg> Neg for Point<T>{
        type Output = Point<T::Output>;
    
        fn neg(self) -> Self::Output {
            Point{
                x: -self.x,
                y: -self.y,
            }
        }
    }

    impl<T: Float> Point<T> {
        pub fn sq_distance(&self, other: &Self) -> T {
            let dx = self.x - other.x;
            let dy = self.y - other.y;
            dx * dx + dy * dy
        }

        pub fn distance(&self, other: &Self) -> T {
            num_traits::Float::sqrt(self.sq_distance(other))
        }

        pub fn aprox_eq(&self, other: &Self) -> bool {
            (self.x - other.x).abs() < T::EPSILON && (self.y - other.y).abs() < T::EPSILON
        }

        pub fn floor(&self) -> Self {
            Self {
                x: self.x.floor(),
                y: self.y.floor(),
            }
        }
    }

    impl<S: num_traits::NumCast> Point<S> {
        pub fn cast<I: num_traits::NumCast>(self) -> Option<Point<I>> {
            num_traits::cast(self.x)
                .and_then(|x| num_traits::cast(self.y).map(|y| Point { x, y }))
        }
    }

    impl<S> Point<S> {
        pub fn as_<I: Copy + 'static>(self) -> Point<I>
        where
            S: AsPrimitive<I>,
        {
            Point {
                x: self.x.as_(),
                y: self.y.as_(),
            }
        }
    }

    impl<T: std::fmt::Display> std::fmt::Display for Point<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "({:2}, {:2})", self.x, self.y)
        }
    }
