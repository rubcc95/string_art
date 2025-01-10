use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use num_traits::AsPrimitive;

    use crate::Float;

    #[derive(Copy, Clone, Debug)]
    pub struct Point<T> {
        pub x: T,
        pub y: T,
    }
    impl<T> Add for Point<T>
    where
        T: Add<Output = T>,
    {
        type Output = Self;
    
        fn add(self, other: Self) -> Self {
            Point {
                x: self.x + other.x,
                y: self.y + other.y,
            }
        }
    }
    
    impl<T> Add<T> for Point<T>
    where
        T: Add<Output = T> + Clone,
    {
        type Output = Self;
    
        fn add(self, scalar: T) -> Self {
            Point {
                x: self.x + scalar.clone(),
                y: self.y + scalar,
            }
        }
    }
    
    impl<T> Sub for Point<T>
    where
        T: Sub<Output = T>,
    {
        type Output = Self;
    
        fn sub(self, other: Self) -> Self {
            Point {
                x: self.x - other.x,
                y: self.y - other.y,
            }
        }
    }
    
    impl<T> Sub<T> for Point<T>
    where
        T: Sub<Output = T> + Clone,
    {
        type Output = Self;
    
        fn sub(self, scalar: T) -> Self {
            Point {
                x: self.x - scalar.clone(),
                y: self.y - scalar,
            }
        }
    }
    
    impl<T> Mul for Point<T>
    where
        T: Mul<Output = T>,
    {
        type Output = Self;
    
        fn mul(self, other: Self) -> Self {
            Point {
                x: self.x * other.x,
                y: self.y * other.y,
            }
        }
    }
    
    impl<T> Mul<T> for Point<T>
    where
        T: Mul<Output = T> + Clone,
    {
        type Output = Self;
    
        fn mul(self, scalar: T) -> Self {
            Point {
                x: self.x * scalar.clone(),
                y: self.y * scalar,
            }
        }
    }
    
    impl<T> Div for Point<T>
    where
        T: Div<Output = T>,
    {
        type Output = Self;
    
        fn div(self, other: Self) -> Self {
            Point {
                x: self.x / other.x,
                y: self.y / other.y,
            }
        }
    }
    
    impl<T> Div<T> for Point<T>
    where
        T: Div<Output = T> + Clone,
    {
        type Output = Self;
    
        fn div(self, scalar: T) -> Self {
            Point {
                x: self.x / scalar.clone(),
                y: self.y / scalar,
            }
        }
    }
    
    impl<T> AddAssign for Point<T>
    where
        T: AddAssign,
    {
        fn add_assign(&mut self, other: Self) {
            self.x += other.x;
            self.y += other.y;
        }
    }
    
    impl<T> AddAssign<T> for Point<T>
    where
        T: AddAssign + Clone,
    {
        fn add_assign(&mut self, scalar: T) {
            self.x += scalar.clone();
            self.y += scalar;
        }
    }
    
    impl<T> SubAssign for Point<T>
    where
        T: SubAssign,
    {
        fn sub_assign(&mut self, other: Self) {
            self.x -= other.x;
            self.y -= other.y;
        }
    }
    
    impl<T> SubAssign<T> for Point<T>
    where
        T: SubAssign + Clone,
    {
        fn sub_assign(&mut self, scalar: T) {
            self.x -= scalar.clone();
            self.y -= scalar;
        }
    }
    
    impl<T> MulAssign for Point<T>
    where
        T: MulAssign,
    {
        fn mul_assign(&mut self, other: Self) {
            self.x *= other.x;
            self.y *= other.y;
        }
    }
    
    impl<T> MulAssign<T> for Point<T>
    where
        T: MulAssign + Clone,
    {
        fn mul_assign(&mut self, scalar: T) {
            self.x *= scalar.clone();
            self.y *= scalar;
        }
    }
    
    impl<T> DivAssign for Point<T>
    where
        T: DivAssign,
    {
        fn div_assign(&mut self, other: Self) {
            self.x /= other.x;
            self.y /= other.y;
        }
    }
    
    impl<T> DivAssign<T> for Point<T>
    where
        T: DivAssign + Clone,
    {
        fn div_assign(&mut self, scalar: T) {
            self.x /= scalar.clone();
            self.y /= scalar;
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
