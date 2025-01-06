use std::path::Path;

use bresenham::Bresenham;
use image::{GenericImageView, Pixel, Rgb, RgbImage};
use num_traits::ToPrimitive;

mod hooks;

pub struct Builder {
    table: Table,
    connections: Vec<Connection>,
}

impl Builder {
    pub fn new(table: Table, initial_hook: usize, direction: HookDirection) -> Self {
        Self {
            table,
            connections: vec![Connection {
                idx: initial_hook,
                dir: direction,
            }],
        }
    }

    pub fn bake(&mut self, line_count: usize) {
        let mut connection = *self.connections.last().unwrap();
        while self.connections.len() < line_count {
            connection = self.table.get_best_connection(connection).unwrap();
            self.connections.push(connection);
        }
    }

    pub fn save_image(&self, file_name: impl AsRef<Path>, scale: f64) {
        let width = (self.table.image.width as f64 * scale) as usize;
        let height = (self.table.image.height as f64 * scale) as usize;
        let mut image =
            RgbImage::from_vec(width as u32, height as u32, vec![255; width * height * 3]).unwrap();

        let mut connections = self.connections.iter();

        if let Some(mut from) = connections.next() {
            while let Some(to) = connections.next() {
                let tangent = tangent_points(
                    self.table.hooks[from.idx],
                    self.table.hooks[to.idx],
                    from.dir,
                    to.dir,
                    self.table.hook_radius,
                )
                .unwrap();
                let s_from = Point {
                    x: scale * tangent.0.x,
                    y: scale * tangent.0.y,
                };
                let s_to = Point {
                    x: scale * tangent.1.x,
                    y: scale * tangent.1.y,
                };
                for point in get_pixels_between(s_from, s_to) {
                    if point.x > 0
                        && point.y > 0
                        && point.x < width as isize
                        && point.y < height as isize
                    {
                        image.put_pixel(point.x as u32, point.y as u32, Rgb([0, 0, 0]));
                    }
                }
                from = to;
            }
        }

        image.save(file_name).unwrap();
    }
}



struct ImageData {
    data: Vec<f64>,
    width: usize,
    height: usize,
}

impl ImageData {
    pub fn new(image: impl GenericImageView) -> Self {
        let (w_pixels, h_pixels) = image.dimensions();
        dbg!(w_pixels, h_pixels);
        Self {
            data: image
            .pixels()
            .map(|(x, y, rgb)| {
                let rgb = rgb.to_rgb();

                let luminance = 0.2126 * rgb[0].to_f64().unwrap()
                    + 0.7152 * rgb[1].to_f64().unwrap()
                    + 0.0722 * rgb[2].to_f64().unwrap();
                255.0 - luminance
            })
            .collect(),
            width: w_pixels as usize,
            height: h_pixels as usize,
        }
    }

    fn get_pixel_mut(&mut self, point: Point<isize>) -> Option<&mut f64> {
        if point.x < 0
            || point.y < 0
            || point.x >= self.width as isize
            || point.y >= self.height as isize
        {
            None
        } else {
            let res = unsafe {
                self.data
                    .get_unchecked_mut(point.y as usize * self.width + point.x as usize)
            };
            Some(res)
        }
    }

    fn get_pixel(&self, point: Point<isize>) -> Option<f64> {
        if point.x < 0
            || point.y < 0
            || point.x >= self.width as isize
            || point.y >= self.height as isize
        {
            None
        } else {
            let res = unsafe {
                self.data
                    .get_unchecked(point.y as usize * self.width + point.x as usize)
            };
            Some(*res)
        }
    }
}

#[derive(Clone, Copy)]
struct Connection {
    idx: usize,
    dir: HookDirection,
}
pub struct Table {
    hooks: Vec<Point<f64>>,
    step: usize,
    image: ImageData,
    hook_radius: f64,
    darkness: f64,
}

impl Table {
    pub fn ellipse(
        image: impl GenericImageView,
        hook_count: usize,
        hook_radius: f64,
        step: usize,
        darkness: f64,
    ) -> Self {
        //let image = image::open(path).unwrap();
        let image = ImageData::new(image);
        //let (w_pixels, h_pixels) = image.dimensions();
        let width = (image.width / 2 - 1) as f64; // Semieje horizontal
        let height = (image.height / 2 - 1) as f64; // Semieje vertical

        Self {
            hooks: (1..hook_count)
                .into_iter()
                .map(|i| {
                    let theta = 2.0 * std::f64::consts::PI * (i as f64) / (hook_count as f64);
                    Point {
                        x: width * (1.0 + theta.cos()) + 0.5,
                        y: height * (1.0 + theta.sin()) + 0.5,
                    }
                })
                .collect(),
            image,
            hook_radius,
            darkness,
            step,
        }
    }

    fn get_best_connection(&mut self, from: Connection) -> Option<Connection> {
        let mut best_weight = f64::NEG_INFINITY;
        let mut best_hook = None;

        let inv_step = self.hooks.len() - self.step;

        for to_idx in 0..self.hooks.len() {
            let distance = from.idx.abs_diff(to_idx);
            if distance > self.step && distance < inv_step {
                let cw_hook = Connection {
                    idx: to_idx,
                    dir: HookDirection::ClockWise,
                };
                let weight_cw = self.get_weight(from, cw_hook);
                if weight_cw > best_weight {
                    best_weight = weight_cw;
                    best_hook = Some(cw_hook);
                }
                let weigth_ccw = self.get_weight(from, cw_hook);
                if weigth_ccw > best_weight {
                    best_weight = weigth_ccw;
                    best_hook = Some(cw_hook);
                }
            }
        }
        let darkness = self.darkness;
        best_hook.map(|hook| {
            let (from, to) = tangent_points(
                self.hooks[from.idx],
                self.hooks[hook.idx],
                from.dir,
                hook.dir,
                self.hook_radius,
            )
            .unwrap();
            for pixel in get_pixels_between(from, to) {
                if let Some(value) = self.image.get_pixel_mut(pixel) {
                    *value -= darkness;
                }
            }
        });

        best_hook
    }

    fn get_weight(&self, from: Connection, to: Connection) -> f64 {
        let from_hook = self.hooks[from.idx];
        let to_hook = self.hooks[to.idx];
        let tangent = tangent_points(from_hook, to_hook, from.dir, to.dir, self.hook_radius);
        match tangent {
            Some((a, b)) => {
                let mut old_weight = 0.0;
                let mut new_weight = 0.0;
                let mut count = 0.0;
                for pixel in get_pixels_between(a, b) {
                    if let Some(value) = self.image.get_pixel(pixel) {
                        if value > 0.0 {
                            old_weight += value;
                            let new_value = value - self.darkness;
                            if new_value > 0.0 {
                                new_weight += new_value;
                            }
                        }
                        count += 1.0;
                    }
                }
                if count == 0.0 {
                    f64::NEG_INFINITY
                } else {
                    (old_weight - new_weight) / count
                }
            }
            None => f64::NEG_INFINITY,
        }
    }
}

#[derive(Copy, Clone, Debug)]
struct Point<T> {
    x: T,
    y: T,
}

struct Line<T> {
    from: Point<T>,
}

#[derive(Clone, Copy)]
pub enum HookDirection {
    ClockWise,
    CounterClockWise,
}

fn get_pixels_between(p1: Point<f64>, p2: Point<f64>) -> impl Iterator<Item = Point<isize>> {
    Bresenham::new(
        (p1.x as isize, p1.y as isize),
        (p2.x as isize, p2.y as isize),
    )
    .map(|(x, y)| Point { x, y })
}

fn tangent_points(
    hook1: Point<f64>,
    hook2: Point<f64>,
    dir1: HookDirection,
    dir2: HookDirection,
    radius: f64,
) -> Option<(Point<f64>, Point<f64>)> {
    let dx = hook2.x - hook1.x;
    let dy = hook2.y - hook1.y;
    let dist = (dx * dx + dy * dy).sqrt();

    // Check if circles are too close or overlapping
    if dist <= f64::EPSILON {
        return None;
    }

    // Determine if we need internal or external tangents
    let sign1 = match dir1 {
        HookDirection::ClockWise => 1.0,
        HookDirection::CounterClockWise => -1.0,
    };
    let sign2 = match dir2 {
        HookDirection::ClockWise => 1.0,
        HookDirection::CounterClockWise => -1.0,
    };

    let r1 = radius;
    let r2 = radius;

    let base_angle = dy.atan2(dx);

    let r = if sign1 * sign2 > 0.0 {
        core::f64::consts::FRAC_PI_2
    } else {
        ((r1 + r2) / dist).acos()
    };

    if r.is_nan() {
        return None;
    }

    let angle1 = base_angle + sign1 * r;
    let angle2 = base_angle + sign2 * r;

    let p1 = Point {
        x: hook1.x + r1 * angle1.cos(),
        y: hook1.y + r1 * angle1.sin(),
    };

    let p2 = Point {
        x: hook2.x + r2 * angle2.cos(),
        y: hook2.y + r2 * angle2.sin(),
    };

    Some((p1, p2))
}

#[test]
fn test() {
    let image = image::open("examples/alba_portrait.jpg").unwrap();
    let a = image.dimensions();
    println!("Dimensions x: {} y: {}", a.0, a.1);
    let image = image.resize(100, 100, image::imageops::FilterType::Nearest);
    // image.get_pixel(Point { x: 10, y: 10 });
    let a = image.dimensions();
    println!("Dimensions x: {} y: {}", a.0, a.1);
    ImageData::new(image);
}
