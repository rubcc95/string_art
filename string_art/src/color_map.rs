pub struct ColorMap<A, C, S> {
    pub anchor: A,
    pub color: C,
    pub weights: Vec<S>,
}

// impl<A: Default> ColorMap<A, C>{
//     pub fn grayscale( color: C, image: Vec<Scalar>) -> Self{

//     }
// }
