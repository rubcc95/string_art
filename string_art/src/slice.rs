use core::slice;
use std::{
    mem::{self, MaybeUninit},
    ptr,
};

pub trait Slice<'s> {
    type Item: 's;
    type Map<'a, S: 'a>: SliceOwner<'a, Item = S>;

    fn len(&self) -> usize;

    fn get(&self, index: usize) -> Option<&Self::Item> {
        if self.len() > index {
            Some(unsafe { self.get_unchecked(index) })
        } else {
            None
        }
    }

    fn get_mut(&mut self, index: usize) -> Option<&mut Self::Item> {
        if self.len() > index {
            Some(unsafe { self.get_unchecked_mut(index) })
        } else {
            None
        }
    }

    unsafe fn get_unchecked(&self, index: usize) -> &Self::Item;

    unsafe fn get_unchecked_mut(&mut self, index: usize) -> &mut Self::Item;

    fn map<'a, S: 'a>(&'a self, f: impl Fn(&'a Self::Item) -> S) -> Self::Map<'a, S> where 's: 'a;

    fn map_mut<'a, S: 'a>(&'a mut self, f: impl Fn(&'a mut Self::Item) -> S) -> Self::Map<'a, S> where 's: 'a;

    fn try_map<'a, S: 'a, E>(
        &self,
        f: impl Fn(&Self::Item) -> Result<S, E>,
    ) -> Result<Self::Map<'a, S>, E>;

    fn raw_slice(&self) -> &[Self::Item];

    fn raw_mut_slice(&mut self) -> &mut [Self::Item];
}

impl<'s, T: 's, const N: usize> Slice<'s> for [T; N] {
    type Item = T;
    type Map<'a, S: 'a> = [S; N];

    fn len(&self) -> usize {
        N
    }

    fn get(&self, index: usize) -> Option<&Self::Item> {
        self.as_slice().get(index)
    }

    fn get_mut(&mut self, index: usize) -> Option<&mut Self::Item> {
        self.as_mut_slice().get_mut(index)
    }

    unsafe fn get_unchecked(&self, index: usize) -> &Self::Item {
        self.as_slice().get_unchecked(index)
    }

    unsafe fn get_unchecked_mut(&mut self, index: usize) -> &mut Self::Item {
        self.as_mut_slice().get_unchecked_mut(index)
    }

    fn map<'a, S: 'a>(&'a self, f: impl Fn(&'a Self::Item) -> S) -> Self::Map<'a, S> where 's: 'a {
        let mut result = DropGuard::new();
        for item in self.into_iter() {
            unsafe { result.push_unchecked(f(item)) }
        }
        unsafe { result.assume_init() }
    }

    fn map_mut<'a, S: 'a>(&'a mut self, f: impl Fn(&'a mut Self::Item) -> S) -> Self::Map<'a, S> where 's: 'a {
        let mut result = DropGuard::new();
        for item in self.into_iter() {
            unsafe { result.push_unchecked(f(item)) }
        }
        unsafe { result.assume_init() }
    }

    fn raw_slice(&self) -> &[Self::Item] {
        self
    }

    fn raw_mut_slice(&mut self) -> &mut [Self::Item] {
        self
    }

    fn try_map<'a, S: 'a, E>(
        &self,
        f: impl Fn(&Self::Item) -> Result<S, E>,
    ) -> Result<Self::Map<'a, S>, E> {
        let mut result = DropGuard::new();
        for item in self {
            match f(item) {
                Ok(val) => unsafe { result.push_unchecked(val) },
                Err(e) => {
                    return Err(e);
                }
            }
        }
        Ok(unsafe { result.assume_init() })
    }
}

impl<'s, T: 's> Slice<'s> for [T] {
    type Item = T;
    type Map<'a, S: 'a> = Vec<S>;

    fn len(&self) -> usize {
        self.len()
    }
    

    fn get(&self, index: usize) -> Option<&Self::Item> {
        self.get(index)
    }

    fn get_mut(&mut self, index: usize) -> Option<&mut Self::Item> {
        self.get_mut(index)
    }

    unsafe fn get_unchecked(&self, index: usize) -> &Self::Item {
        self.get_unchecked(index)
    }

    unsafe fn get_unchecked_mut(&mut self, index: usize) -> &mut Self::Item {
        self.get_unchecked_mut(index)
    }

    fn map<'a, S: 'a>(&'a self, f: impl Fn(&'a Self::Item) -> S) -> Self::Map<'a, S> where 's: 'a{
        self.iter().map(f).collect()
    }

    fn map_mut<'a, S: 'a>(&'a mut self, f: impl Fn(&'a mut Self::Item) -> S) -> Self::Map<'a, S> where 's: 'a{
        self.iter_mut().map(f).collect()
    }

    fn raw_slice(&self) -> &[Self::Item] {
        self
    }

    fn raw_mut_slice(&mut self) -> &mut [Self::Item] {
        self
    }

    fn try_map<'a, S: 'a, E>(
        &self,
        f: impl Fn(&Self::Item) -> Result<S, E>,
    ) -> Result<Self::Map<'a, S>, E> {
        self.iter().map(f).collect()
    }
}

pub trait SliceOwner<'s>: IntoIterator<Item: 's> {
    type Map<'a, S: 'a>: SliceOwner<'a, Item = S>;
    type Slice: ?Sized + Slice<'s, Item = Self::Item>;

    fn len(&self) -> usize;

    fn as_slice(&self) -> &Self::Slice;

    fn as_mut_slice(&mut self) -> &mut Self::Slice;

    fn map<'a, S>(self, f: impl Fn(Self::Item) -> S) -> Self::Map<'a, S>;

    fn try_map<'a, S, E>(
        self,
        f: impl Fn(Self::Item) -> Result<S, E>,
    ) -> Result<Self::Map<'a, S>, E>;
}

impl<'s, T: 's, const N: usize> SliceOwner<'s> for [T; N] {
    type Map<'a, S: 'a> = [S; N];
    type Slice = Self;

    fn len(&self) -> usize {
        N
    }

    fn as_slice(&self) -> &Self {
        self
    }

    fn as_mut_slice(&mut self) -> &mut Self {
        self
    }

    fn map<'a, S: 'a>(self, f: impl Fn(T) -> S) -> Self::Map<'a, S> {
        self.map(f)
    }

    fn try_map<'a, S: 'a, E>(
        self,
        f: impl Fn(Self::Item) -> Result<S, E>,
    ) -> Result<Self::Map<'a, S>, E> {
        let mut result = DropGuard::new();
        for item in self.into_iter() {
            match f(item) {
                Ok(val) => unsafe { result.push_unchecked(val) },
                Err(e) => {
                    return Err(e);
                }
            }
        }
        Ok(unsafe { result.assume_init() })
    }
}

impl<'s, T: 's> SliceOwner<'s> for Vec<T> {
    type Map<'a, S: 'a> = Vec<S>;
    type Slice = [T];

    fn len(&self) -> usize {
        self.len()
    }

    fn as_slice(&self) -> &[T] {
        self
    }

    fn as_mut_slice(&mut self) -> &mut [T] {
        self
    }

    fn map<'a, S: 'a>(self, f: impl Fn(T) -> S) -> Self::Map<'a, S> {
        self.into_iter().map(f).collect()
    }

    fn try_map<'a, S: 'a, E>(
        self,
        f: impl Fn(Self::Item) -> Result<S, E>,
    ) -> Result<Self::Map<'a, S>, E> {
        self.into_iter().map(f).collect()
    }
}

struct DropGuard<T, const N: usize> {
    items: MaybeUninit<[T; N]>,
    len: usize,
}

impl<T, const N: usize> DropGuard<T, N> {
    pub fn new() -> Self {
        Self {
            items: MaybeUninit::uninit(),
            len: 0,
        }
    }

    pub unsafe fn push_unchecked(&mut self, item: T) {
        (self.items.as_mut_ptr() as *mut T)
            .add(self.len)
            .write(item);
        self.len = self.len.unchecked_add(1);
    }

    pub unsafe fn assume_init(self) -> [T; N] {
        let res = ptr::read(&self.items).assume_init();
        mem::forget(self);
        res
    }
}

impl<T, const N: usize> Drop for DropGuard<T, N> {
    fn drop(&mut self) {
        unsafe {
            ptr::drop_in_place(slice::from_raw_parts_mut(self.items.as_mut_ptr(), self.len))
        }
    }
}
