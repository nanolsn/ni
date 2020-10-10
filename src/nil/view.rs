/// Immutable array.
#[derive(Clone)]
pub struct View<T>(Option<Box<[T]>>);

impl<T> View<T> {
    pub fn from_vec(vec: Vec<T>) -> Self {
        if vec.is_empty() {
            Self::empty()
        } else {
            Self(Some(vec.into_boxed_slice()))
        }
    }

    pub fn from_box(boxed: Box<[T]>) -> Self {
        if boxed.is_empty() {
            Self::empty()
        } else {
            Self(Some(boxed))
        }
    }

    pub fn empty() -> Self {
        Self(None)
    }
}

impl<T> Default for View<T> {
    fn default() -> Self {
        Self::empty()
    }
}

impl<T> std::ops::Deref for View<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        if let Some(view) = &self.0 {
            view.as_ref()
        } else {
            &[]
        }
    }
}

impl<T> AsRef<[T]> for View<T> {
    fn as_ref(&self) -> &[T] {
        &*self
    }
}

use std::fmt::{Debug, Formatter, Result};

impl<T> Debug for View<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{:?}", self.as_ref())
    }
}

impl<T> From<Vec<T>> for View<T> {
    fn from(vec: Vec<T>) -> Self {
        Self::from_vec(vec)
    }
}

impl<T> From<Box<[T]>> for View<T> {
    fn from(boxed: Box<[T]>) -> Self {
        Self::from_box(boxed)
    }
}
