use super::*;

#[test]
#[cfg(feature = "w32")]
fn op_size_of() {
    assert_eq!(std::mem::size_of::<Op>(), 32)
}

#[test]
#[cfg(feature = "w64")]
fn op_size_of() {
    assert_eq!(std::mem::size_of::<Op>(), 64)
}
