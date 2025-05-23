// use std::{collections::HashMap, slice::SliceIndex};

// use num_traits::{AsPrimitive, ConstZero as _};

// use crate::{
//     geometry::{Point, Rect},
//     nails::{self, Links as _},
//     verboser, BakedSegment, Float,
// };

// use super::NailTable;

// pub struct Rectangle<N: nails::Handle> {
//     nails: Vec<N::Nail>,
//     handle: N,
//     segments: HashMap<SegmentKey<N::Link>, BakedSegment<N::Scalar>>,
//     width: usize,
//     height: usize,
// }

// #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
// pub struct SegmentKey<L>{
//     from: (Id, L),
//     to: (Id, L)
// }

// #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
// pub enum Id {
//     TopLeft,
//     Top(usize),
//     TopRight,
//     Right(usize),
//     BotRight,
//     Bot(usize),
//     BotLeft,
//     Left(usize),
// }



// impl Default for Id{
//     fn default() -> Self {
//         Self::TopLeft
//     }
// }

// impl core::fmt::Display for Id {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Self::TopLeft => write!(f, "Top-Left"),
//             Self::Top(arg0) => write!(f, "Top-{}", arg0),
//             Self::TopRight => write!(f, "Top-Right"),
//             Self::Right(arg0) => write!(f, "Right-{}", arg0),
//             Self::BotRight => write!(f, "Bot-Right"),
//             Self::Bot(arg0) => write!(f, "Bot-{}", arg0),
//             Self::BotLeft => write!(f, "Bot-Left"),
//             Self::Left(arg0) => write!(f, "Left-{}", arg0),
//         }
//     }
// }

// impl<N: nails::Handle> Rectangle<N> {
//     pub fn new<B: nails::Builder<Scalar: Float, Handle = N, Nail = N::Nail>>(
//         rect: Rect<B::Scalar>,
//         nail_builder: B,
//         nail_count: usize,
//         verboser: &mut impl verboser::Verboser,
//     ) -> Result<Self, Error<N::Error>>
//     where
//         usize: AsPrimitive<B::Scalar>,
//     {
//         if nail_count == 0 {
//             return Err(Error::MinNailCount);
//         }
//         if nail_count % 4 != 0 {
//             return Err(Error::NotMultipleOf4);
//         }

//         let width = num_traits::ToPrimitive::to_usize(&num_traits::Float::round(
//             ((nail_count + 1).as_() * rect.width - rect.height) / (rect.width + rect.height),
//         ))
//         .unwrap_or(0)
//             + 2;
//         let height = nail_count + 4 - width;
//         Self::new_with_exact_count(rect, nail_builder, width, height, verboser).map_err(Error::Nail)
//     }

//     pub fn new_with_exact_count<B: nails::Builder<Scalar: Float, Handle = N, Nail = N::Nail>>(
//         rect: Rect<B::Scalar>,
//         nail_builder: B,
//         width: usize,
//         height: usize,
//         _: &mut impl verboser::Verboser,
//     ) -> Result<Self, N::Error>
//     where
//         usize: AsPrimitive<B::Scalar>,
//     {
//         let mut nails = Vec::new();
//         let size: Rect<B::Scalar> = rect.as_();
//         let step: Point<B::Scalar> = Point {
//             x: size.width / (width + 1).as_(),
//             y: size.height / (height + 1).as_(),
//         };

//         nails.push(nail_builder.build_nail(
//             Point {
//                 x: B::Scalar::ZERO,
//                 y: B::Scalar::ZERO,
//             },
//             B::Scalar::FRAC_5PI_4,
//         ));
//         nails.extend((1..=width).map(|idx| {
//             nail_builder.build_nail(
//                 Point {
//                     x: step.x * idx.as_(),
//                     y: B::Scalar::ZERO,
//                 },
//                 B::Scalar::FRAC_3PI_2,
//             )
//         }));
//         nails.push(nail_builder.build_nail(
//             Point {
//                 x: size.width,
//                 y: B::Scalar::ZERO,
//             },
//             B::Scalar::FRAC_7PI_4,
//         ));
//         nails.extend((1..=height).map(|idx| {
//             nail_builder.build_nail(
//                 Point {
//                     x: size.width,
//                     y: step.y * idx.as_(),
//                 },
//                 B::Scalar::ZERO,
//             )
//         }));
//         nails.push(nail_builder.build_nail(
//             Point {
//                 x: size.width,
//                 y: size.height,
//             },
//             B::Scalar::FRAC_PI_4,
//         ));
//         nails.extend((1..=width).rev().map(|idx| {
//             nail_builder.build_nail(
//                 Point {
//                     x: step.x * idx.as_(),
//                     y: size.height,
//                 },
//                 B::Scalar::FRAC_PI_2,
//             )
//         }));
//         nails.push(nail_builder.build_nail(
//             Point {
//                 x: B::Scalar::ZERO,
//                 y: size.height,
//             },
//             B::Scalar::FRAC_3PI_4,
//         ));
//         nails.extend((1..=height).rev().map(|idx| {
//             nail_builder.build_nail(
//                 Point {
//                     x: B::Scalar::ZERO,
//                     y: step.y * idx.as_(),
//                 },
//                 B::Scalar::PI,
//             )
//         }));

//         fn build_segments<
//             N: nails::Handle,
//             R: SliceIndex<[N::Nail], Output = [N::Nail]> + Clone,
//         >(
//             segments: &mut Vec<BakedSegment<N::Scalar>>,
//             handle: &N,
//             nails: &[N::Nail],
//             from_nail: usize,
//             range: R,
//         ) -> Result<(), N::Error> {
//             for from_link in N::LINKS {
//                 for to_nail in unsafe { nails.get_unchecked(range.clone()) } {
//                     for to_link in N::LINKS {
//                         segments.push(BakedSegment::from(handle.get_segment(
//                             (unsafe { nails.get_unchecked(from_nail) }, from_link),
//                             (to_nail, to_link),
//                         )?))
//                     }
//                 }
//             }
//             Ok(())
//         }

//         let handle = nail_builder.build_handle();
//         let mut segments = HashMap::new();

//         build_segments(
//             &mut segments,
//             &handle,
//             &nails,
//             0,
//             (width + 2)..(2 * width + height + 3),
//         )?; 
//         for from_nail in 1..=width {
//             build_segments(&mut segments, &handle, &nails, from_nail, (width + 2)..)?;
//         }
//         build_segments(
//             &mut segments,
//             &handle,
//             &nails,
//             width + 1,
//             (width + height + 3)..,
//         )?;
//         for from_nail in (width + 2)..(width + height + 2) {
//             build_segments(
//                 &mut segments,
//                 &handle,
//                 &nails,
//                 from_nail,
//                 (width + height + 3)..,
//             )?;
//         }
//         build_segments(
//             &mut segments,
//             &handle,
//             &nails,
//             width + height + 2,
//             (2 * width + height + 4)..,
//         )?;
//         for from_nail in (width + height + 3)..(2 * width + height + 3) {
//             build_segments(
//                 &mut segments,
//                 &handle,
//                 &nails,
//                 from_nail,
//                 (2 * width + height + 4)..,
//             )?;
//         }
//         Ok(Self {
//             nails,
//             handle,
//             segments,
//             width,
//             height,
//         })
//     }

//     fn top_left_idx(&self, link: N::Link) -> usize {
//         link.into() * (self.width + self.height + 1) * N::Links::LEN
//     }

//     fn top_idx(&self, idx: usize, link: N::Link) -> usize {
//         N::Links::SQ_LEN * (self.width + self.height + 1)
//             + (idx * N::Links::LEN + link.into())
//                 * N::Links::LEN
//                 * (self.width + 2 * self.height + 2)
//     }

//     fn top_right_idx(&self, link: N::Link) -> usize {
//         N::Links::SQ_LEN * (self.width * (self.width + 2 * self.height + 3) + self.height + 1)
//             + N::Links::LEN * (self.width + self.height + 1) * link.into()
//     }

//     fn right_idx(&self, idx: usize, link: N::Link) -> usize {
//         N::Links::SQ_LEN
//             * (self.width * (self.width + 2 * self.height + 3) + 2 * self.height + self.width + 2)
//             + (idx * N::Links::LEN + link.into()) * N::Links::LEN * (self.width + self.height + 1)
//     }

//     fn bot_right_idx(&self, link: N::Link) -> usize {
//         // Computing these constant values each iteration is faster than storing them in memory for the whole structure.
//         // Maybe access to memory in this situation is slower than CPU registry calculations?
//         // Maybe a fantastic compiler optimization?
//         // Not sure since ASM has not been checked for.
//         N::Links::SQ_LEN
//             * (self.width * self.width
//                 + self.height * self.height
//                 + 3 * self.width * self.height
//                 + 4 * self.width
//                 + 3 * self.height
//                 + 2)
//         //
//             + N::Links::LEN * self.height * link.into()
//     }

//     pub fn bot_idx(&self, idx: usize, link: N::Link) -> usize {
//         N::Links::SQ_LEN
//             * (self.width * self.width
//                 + self.height * self.height
//                 + 3 * self.width * self.height
//                 + 4 * self.width
//                 + 4 * self.height
//                 + 2)
//             + (idx * N::Links::LEN + link.into()) * N::Links::LEN * self.height
//     }
// }

// unsafe impl<N: nails::Handle> NailTable for Rectangle<N> {
//     type Handle = N;

//     type Id = Id;

//     fn nails(&self) -> &[<Self::Handle as nails::Handle>::Nail] {
//         &self.nails
//     }

//     unsafe fn comb_count(&self, nail_idx: Self::Id) -> usize {
//         match nail_idx {
//             Id::Top(_) | Id::Bot(_) => 2 + self.width + 2 * self.height,
//             Id::Right(_) | Id::Left(_) => 2 + self.height + 2 * self.width,
//             _ => self.height + self.width + 1,
//         }
//     }

//     unsafe fn get_segment(
//         &mut self,
//         nail_idx: Self::Id,
//         link: <Self::Handle as nails::Handle>::Link,
//         offset: usize,
//         other_link: <Self::Handle as nails::Handle>::Link,
//     ) -> (
//         Self::Id,
//         &mut BakedSegment<<Self::Handle as nails::Handle>::Scalar>,
//     ) {
//         todo!()
//     }

//     fn is_valid(&self, nail_idx: Self::Id) -> bool {
//         match nail_idx {
//             Id::Top(id) | Id::Bot(id) => id < self.width,
//             Id::Right(id) | Id::Left(id) => id < self.height,
//             _ => true,
//         }
//     }

//     fn handle(&self) -> &Self::Handle {
//         &self.handle
//     }
// }

// #[derive(Debug, thiserror::Error)]
// pub enum Error<N> {
//     #[error("Nail count must be greater or equal to 4")]
//     MinNailCount,
//     #[error("Nail count must be multiple of 4")]
//     NotMultipleOf4,
//     #[error(transparent)]
//     Nail(N),
// }

// #[cfg(test)]
// mod test {
//     use crate::{
//         geometry::Rect,
//         nails::{self, circular::Direction},
//         verboser, NailTable,
//     };

//     use super::{Rectangle, Id};

//     #[test]
//     fn test() {
//         let mut table = Rectangle::new_with_exact_count(
//             Rect::new(6.0, 5.0),
//             nails::Circular::<f32>::new(0.0),
//             4,
//             5,
//             &mut verboser::Silent,
//         )
//         .unwrap();
//         println!(
//             "nails: {}, segments: {}",
//             table.nails.len(),
//             table.segments.len()
//         );
//         println!();
//         for (idx, point) in table.nails.iter().enumerate() {
//             println!("{}: {}", idx, point);
//         }
//         println!();
//         unsafe {
//             for index in 0..table.comb_count(Id::Right(2)) {
//                 let a = table.get_segment(
//                     Id::Right(2),
//                     Direction::COUNTER_CLOCK_WISE,
//                     index,
//                     Direction::CLOCK_WISE,
//                 );
//                 println!("{}: point: {}, segment: {} ", index, a.0, a.1.segment);
//             }
//         }
//     }
// }
