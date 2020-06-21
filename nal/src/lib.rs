mod layout;
mod layout_builder;
mod view;

pub use layout::*;
pub use layout_builder::*;

pub const fn size_of<T>() -> common::UWord { std::mem::size_of::<T>() as common::UWord }

const WORD_SIZE: common::UWord = size_of::<common::UWord>();
