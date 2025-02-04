use num_traits::AsPrimitive;

use crate::{geometry::Point, Float, image::PixelData};

pub fn blur<S: Float>(image: &PixelData<S>, radius: usize) -> Vec<S>
where
    usize: AsPrimitive<S>,
{
    let kernel = create_kernel::<S>(radius);
    let mut output = vec![S::ZERO; image.pixels().len()];
    let width = image.width;
    let height = image.height;

    for i in 0..height {
        for j in 0..width {
            let mut sum = S::ZERO;
            for k in 0..kernel.len() {
                let x = (j + k).saturating_sub(radius).min(width - 1);
                sum += *unsafe { image.get_unchecked(Point { x, y: i }) } * kernel[k];
            }
            output[i * width + j] = sum;
        }
    }

    for j in 0..width {
        for i in 0..height {
            let mut sum = S::ZERO;
            for k in 0..kernel.len() {
                let y = (i + k).saturating_sub(radius).min(height - 1);
                sum += output[y as usize * width + j] * kernel[k];
            }
            output[i * width + j] = sum;
        }
    }

    output
}

fn create_kernel<S: Float>(len: usize) -> Vec<S>
where
    usize: AsPrimitive<S>,
{
    if len == 0 {
        return vec![S::ONE];
    }
    let sum = 2_usize.pow(unsafe { (len as u32).unchecked_sub(1) }).as_();
    (0..=2 * len)
        .map(|i| sum / (2_usize.pow(i.abs_diff(len) as u32).as_() * (S::THREE * sum - S::ONE)))
        .collect()
}
