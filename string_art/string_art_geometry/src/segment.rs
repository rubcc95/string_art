use super::Point;
use bresenham::Bresenham;
use num_traits::*;
use std::{fmt, ops::*};

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Segment<T> {
    pub start: Point<T>,
    pub end: Point<T>,
}

impl<T> Add for Segment<T>
where
    T: Add<Output = T>,
{
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Segment {
            start: self.start + other.start,
            end: self.end + other.end,
        }
    }
}

impl<T> Add<T> for Segment<T>
where
    T: Add<Output = T> + Clone,
{
    type Output = Self;

    fn add(self, scalar: T) -> Self {
        Segment {
            start: self.start + scalar.clone(),
            end: self.end + scalar,
        }
    }
}

impl<T> Sub for Segment<T>
where
    T: Sub<Output = T>,
{
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Segment {
            start: self.start - other.start,
            end: self.end - other.end,
        }
    }
}

impl<T> Sub<T> for Segment<T>
where
    T: Sub<Output = T> + Clone,
{
    type Output = Self;

    fn sub(self, scalar: T) -> Self {
        Segment {
            start: self.start - scalar.clone(),
            end: self.end - scalar,
        }
    }
}

impl<T> Mul for Segment<T>
where
    T: Mul<Output = T>,
{
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Segment {
            start: self.start * other.start,
            end: self.end * other.end,
        }
    }
}

impl<T> Mul<T> for Segment<T>
where
    T: Mul<Output = T> + Clone,
{
    type Output = Self;

    fn mul(self, scalar: T) -> Self {
        Segment {
            start: self.start * scalar.clone(),
            end: self.end * scalar,
        }
    }
}

impl<T> Div for Segment<T>
where
    T: Div<Output = T>,
{
    type Output = Self;

    fn div(self, other: Self) -> Self {
        Segment {
            start: self.start / other.start,
            end: self.end / other.end,
        }
    }
}

impl<T> Div<T> for Segment<T>
where
    T: Div<Output = T> + Clone,
{
    type Output = Self;

    fn div(self, scalar: T) -> Self {
        Segment {
            start: self.start / scalar.clone(),
            end: self.end / scalar,
        }
    }
}

impl<T> AddAssign for Segment<T>
where
    T: AddAssign,
{
    fn add_assign(&mut self, other: Self) {
        self.start += other.start;
        self.end += other.end;
    }
}

impl<T> AddAssign<T> for Segment<T>
where
    T: AddAssign + Clone,
{
    fn add_assign(&mut self, scalar: T) {
        self.start += scalar.clone();
        self.end += scalar;
    }
}

impl<T> SubAssign for Segment<T>
where
    T: SubAssign,
{
    fn sub_assign(&mut self, other: Self) {
        self.start -= other.start;
        self.end -= other.end;
    }
}

impl<T> SubAssign<T> for Segment<T>
where
    T: SubAssign + Clone,
{
    fn sub_assign(&mut self, scalar: T) {
        self.start -= scalar.clone();
        self.end -= scalar;
    }
}

impl<T> MulAssign for Segment<T>
where
    T: MulAssign,
{
    fn mul_assign(&mut self, other: Self) {
        self.start *= other.start;
        self.end *= other.end;
    }
}

impl<T> MulAssign<T> for Segment<T>
where
    T: MulAssign + Clone,
{
    fn mul_assign(&mut self, scalar: T) {
        self.start *= scalar.clone();
        self.end *= scalar;
    }
}

impl<T> DivAssign for Segment<T>
where
    T: DivAssign,
{
    fn div_assign(&mut self, other: Self) {
        self.start /= other.start;
        self.end /= other.end;
    }
}

impl<T> DivAssign<T> for Segment<T>
where
    T: DivAssign + Clone,
{
    fn div_assign(&mut self, scalar: T) {
        self.start /= scalar.clone();
        self.end /= scalar;
    }
}

impl<T: Neg> Neg for Segment<T> {
    type Output = Segment<T::Output>;

    fn neg(self) -> Self::Output {
        Segment {
            start: -self.start,
            end: -self.end,
        }
    }
}

impl<T: fmt::Display> fmt::Display for Segment<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{:2}, {:2}]", self.start, self.end)
    }
}

impl<T> Segment<T> {
    pub fn new(start: Point<T>, end: Point<T>) -> Self {
        Self { start, end }
    }
}

impl<T: num_traits::NumCast> Segment<T> {
    pub fn cast<I: num_traits::NumCast>(self) -> Option<Segment<I>> {
        self.start
            .cast()
            .and_then(|start| self.end.cast().map(|end| Segment { start, end }))
    }
}

impl<T> Segment<T> {
    pub fn as_<S: Copy + 'static>(self) -> Segment<S>
    where
        T: AsPrimitive<S>,
    {
        Segment {
            start: self.start.as_(),
            end: self.end.as_(),
        }
    }
}

impl Segment<isize> {
    pub fn bresenham(&self) -> impl Iterator<Item = Point<isize>> + use<> {
        Bresenham::new((self.start.x, self.start.y), (self.end.x, self.end.y)).map(Point::from)
    }
}

impl<T: Float + 'static> Segment<T> {
    pub fn floor(&self) -> Self {
        Self {
            start: self.start.floor(),
            end: self.end.floor(),
        }
    }
}
