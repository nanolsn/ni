/// Immutable array.
#[derive(Clone)]
pub struct View<T>(Box<[T]>);

impl<T> View<T> {
    pub fn from_vec(vec: Vec<T>) -> Self { Self(vec.into_boxed_slice()) }

    pub fn from_box(boxed: Box<[T]>) -> Self { Self(boxed) }
}

impl<T> std::ops::Deref for View<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target { &self.0 }
}

impl<T> AsRef<[T]> for View<T> {
    fn as_ref(&self) -> &[T] { &*self }
}

use std::fmt::{Debug, Formatter, Result};

impl<T> Debug for View<T>
    where
        T: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result { write!(f, "{:?}", self.0) }
}

impl<T> From<Vec<T>> for View<T> {
    fn from(vec: Vec<T>) -> Self { Self::from_vec(vec) }
}

impl<T> From<Box<[T]>> for View<T> {
    fn from(boxed: Box<[T]>) -> Self { Self::from_box(boxed) }
}
