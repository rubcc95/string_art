use std::{iter::FusedIterator, ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign}};

use super::Point;
use crate::Float;
use bresenham::Bresenham;
use image::GenericImage;
use num_traits::AsPrimitive;
use std::fmt;

#[derive(Clone, Copy)]
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

impl<T: Neg> Neg for Segment<T>{
    type Output = Segment<T::Output>;

    fn neg(self) -> Self::Output {
        Segment{            
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

impl<T: Float> Segment<T> {
    pub fn intersection(&self, other: &Segment<T>) -> Option<Point<T>> {
        let p = self.start;
        let q = other.start;
        let r = Point {
            x: self.end.x - self.start.x,
            y: self.end.y - self.start.y,
        };
        let s = Point {
            x: other.end.x - other.start.x,
            y: other.end.y - other.start.y,
        };

        let rxs = r.x * s.y - r.y * s.x;
        let qmp = Point {
            x: q.x - p.x,
            y: q.y - p.y,
        };
        let qpxr = qmp.x * r.y - qmp.y * r.x;

        if rxs.abs() < T::epsilon() {
            // Los segmentos son colineales o paralelos
            return None;
        }

        let t = (qmp.x * s.y - qmp.y * s.x) / rxs;
        let u = qpxr / rxs;

        if (T::ZERO..=T::ONE).contains(&t) && (T::ZERO..=T::ONE).contains(&u) {
            Some(Point {
                x: p.x + t * r.x,
                y: p.y + t * r.y,
            })
        } else {
            None
        }
    }

    pub fn is_m_positive(&self) -> bool {
        let dx = self.end.x - self.start.x;
        if dx.abs() < T::EPSILON {
            return true; // Pendiente infinita
        }
        let dy = self.end.y - self.start.y;

        dy.signum() == dx.signum()
    }

    pub fn draw<I: GenericImage>(&self, img: &mut I, pixel: I::Pixel) {
        for point in self.cast().unwrap().points_between() {
            if point.x >= 0
                && point.y >= 0
                && point.x < img.width() as isize
                && point.y < img.height() as isize
            {
                img.put_pixel(point.x as u32, point.y as u32, pixel);
            }
        }
    }

    pub fn parallel_at_distance(&self, distance: T) -> Self {
        let dx = self.end.x - self.start.x;
        let dy = self.end.y - self.start.y;
        let length = num_traits::Float::sqrt(dx * dx + dy * dy);
        let ux = dx / length;
        let uy = dy / length;

        let offset_x = -uy * distance;
        let offset_y = ux * distance;

        let new_start = Point {
            x: self.start.x + offset_x,
            y: self.start.y + offset_y,
        };
        let new_end = Point {
            x: self.end.x + offset_x,
            y: self.end.y + offset_y,
        };

        Segment::new(new_start, new_end)
    }

    pub fn floor(&self) -> Self {
        Self {
            start: self.start.floor(),
            end: self.end.floor(),
        }
    }
}

impl Segment<isize> {
    pub fn points_between(&self) -> impl Iterator<Item = Point<isize>> {
        Bresenham::new((self.start.x, self.start.y), (self.end.x, self.end.y))
            .map(|(x, y)| Point { x, y })
    }
}

pub trait IntoSegments<T> {
    type Iterator: Iterator<Item = Segment<T>>;
    fn into_edges(self) -> Self::Iterator;
}

impl<'a, T: Copy> IntoSegments<T> for &'a [Point<T>] {
    type Iterator = PointsIterator<'a, T>;
    fn into_edges(self) -> PointsIterator<'a, T> {
        PointsIterator::new(self)
    }
}

pub struct PointsIterator<'a, T> {
    curr: usize,
    points: &'a [Point<T>],
}

impl<'a, T> PointsIterator<'a, T> {
    fn new(points: &'a [Point<T>]) -> Self {
        Self { curr: 0, points }
    }
}

impl<'a, T: Copy> Iterator for PointsIterator<'a, T> {
    type Item = Segment<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr < self.points.len() {
            unsafe {
                let start = *self.points.get_unchecked(self.curr);
                self.curr = self.curr.unchecked_add(1);
                Some(Segment::new(
                    start,
                    *self.points.get_unchecked(if self.curr < self.points.len() {
                        self.curr
                    } else {
                        0
                    }),
                ))
            }
        } else {
            None
        }
    }
}

impl<'a, T: Copy> DoubleEndedIterator for PointsIterator<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.curr > 0 {
            unsafe {
                let start = *self.points.get_unchecked(self.curr);
                self.curr = self.curr.unchecked_sub(1);
                Some(Segment::new(
                    start,
                    *self.points.get_unchecked(if self.curr > 0 {
                        self.curr
                    } else {
                        self.points.len().unchecked_sub(1)
                    }),
                ))
            }
        } else {
            None
        }
    }
}

impl<'a, T: Copy> ExactSizeIterator for PointsIterator<'a, T> {
    fn len(&self) -> usize {
        self.points.len()
    }
}

impl<'a, T> FusedIterator for PointsIterator<'a, T> where T: Copy {}
