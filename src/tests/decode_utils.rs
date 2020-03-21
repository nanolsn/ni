use crate::decode_utils::*;

#[test]
fn decode_test() {
    assert_eq!(decode_x(0x00), 0);
    assert_eq!(decode_x(0x01), 1);

    assert_eq!(decode_xy(0x00), (0, 0));
    assert_eq!(decode_xy(0x01), (1, 0));
    assert_eq!(decode_xy(0b0000_1011), (3, 2));

    assert_eq!(decode_xyz(0b0010_0111), (3, 1, 2));
}

#[test]
fn decode_value_test() {
    let mut code = vec![0x05_u8, 0xFF].into_iter();
    let value = decode(&mut code).unwrap();

    assert_eq!(value, 5);
    assert_eq!(code.next(), Some(0xFF));

    let mut code = vec![0b1000_0000_u8, 0x05, 0xFF].into_iter();
    let value = decode(&mut code).unwrap();

    assert_eq!(value, 5);
    assert_eq!(code.next(), Some(0xFF));

    let mut code = vec![0b1000_0010_u8, 0xFF, 0xFF, 0x10, 0x00].into_iter();
    let value = decode(&mut code).unwrap();

    assert_eq!(value, 0x10FFFF);
    assert_eq!(code.next(), None);
}
