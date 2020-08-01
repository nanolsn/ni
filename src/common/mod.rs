#[cfg(test)]
mod tests;

pub mod bits;
mod expected;
pub mod op_codes;
mod operations;

pub use expected::*;
pub use operations::*;

#[cfg(feature = "w32")]
pub type UWord = u32;
#[cfg(feature = "w32")]
pub type IWord = i32;

#[cfg(feature = "w64")]
pub type UWord = u64;
#[cfg(feature = "w64")]
pub type IWord = i64;
