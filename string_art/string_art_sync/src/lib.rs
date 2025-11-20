#[cfg(not(target_arch = "wasm32"))]
pub use from_rayon::*;
#[cfg(target_arch = "wasm32")]
pub use from_workers::*;

#[cfg(not(target_arch = "wasm32"))]
mod from_rayon {
    use rayon::prelude::*;

    pub use core::marker::Send as CondSend;
    pub use core::marker::Sync as CondSync;

    pub fn for_each<T: CondSend>(bufs: &mut [T], f: impl Fn(usize, &mut T) + CondSync) {
        bufs.par_iter_mut()
            .enumerate()
            .for_each(|(index, t)| f(index, t));
    }

    pub fn cpus() -> usize {
        num_cpus::get()
    }
}

#[cfg(target_arch = "wasm32")]
mod from_workers {
    pub trait CondSync {}

    impl<T: ?Sized> CondSync for T {}

    pub trait CondSend {}

    impl<T: ?Sized> CondSend for T {}

    pub fn for_each<T: CondSend>(buf: &mut [T], f: impl Fn(usize, &mut T) + CondSync) {
        buf.iter_mut()
            .enumerate()
            .for_each(|(index, t)| f(index, t));
    }

    pub const fn cpus() -> usize {
        1
    }
}
