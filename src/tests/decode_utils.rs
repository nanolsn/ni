use crate::decode_utils::*;
use crate::operation::*;

#[test]
fn decode_test() {
    assert_eq!(decode_x(0b1000_0100), 0);
    assert_eq!(decode_x(0b0010_0001), 1);

    assert_eq!(decode_y(0b1000_0001), 0);
    assert_eq!(decode_y(0b0010_0100), 1);

    assert_eq!(decode_z(0b1000_0100), 0);
    assert_eq!(decode_z(0b0010_0001), 2);

    assert_eq!(decode_w(0b1000_0001), 2);
    assert_eq!(decode_w(0b0010_0100), 0);
}

#[test]
fn decode_value_test() {
    let mut code = vec![0x05_u8].into_iter();
    let value = decode(&mut code).unwrap();

    assert_eq!(value, 5);
    assert!(code.next().is_none());

    let mut code = vec![0b1000_0000_u8, 0x05].into_iter();
    let value = decode(&mut code).unwrap();

    assert_eq!(value, 5);
    assert!(code.next().is_none());

    let mut code = vec![0b1000_0010_u8, 0xFF, 0xFF, 0x10, 0x00].into_iter();
    let value = decode(&mut code).unwrap();

    assert_eq!(value, 0x10FFFF);
    assert!(code.next().is_none());
}

#[test]
fn decode_op_test() {
    let mut code = vec![
        0x00_u8, // nop
        0x01, 0b0001_0000, // stop ref(16)
        0x01, 0b1100_0000, 0xA, // stop const(10)
        0x01, 0b1000_0000, 0x0, // stop ref(0)
        0x02, 0b1100_0000, 0x1, // wait const(1)
        0x03, 0b1101_0000, 0x0, 0x7, // set b1 ret(0) const(7)
        0x03, 0b0001_0100, 0x1, 0x0, 0x1, // set b2 ref(1) ref(256)
        0x03, 0b1001_1000, 0x5, 0x1, 0x1, 0x0, 0x0, // set b4 ref(5) const(257)
    ].into_iter();

    let nop = decode_op(&mut code).unwrap();
    assert_eq!(nop, Op::Nop);

    let stop = decode_op(&mut code).unwrap();
    assert_eq!(stop, Op::Stop(Value::Ref(16)));

    let stop = decode_op(&mut code).unwrap();
    assert_eq!(stop, Op::Stop(Value::Const(10)));

    let stop = decode_op(&mut code).unwrap();
    assert_eq!(stop, Op::Stop(Value::Ref(0)));

    let wait = decode_op(&mut code).unwrap();
    assert_eq!(wait, Op::Wait(Value::Const(1)));

    let set = decode_op(&mut code).unwrap();
    assert_eq!(set, Op::Set(RefRet::Return(0), Value::Const(7), 2));

    let set = decode_op(&mut code).unwrap();
    assert_eq!(set, Op::Set(RefRet::Ref(1), Value::Ref(256), 2));
}
