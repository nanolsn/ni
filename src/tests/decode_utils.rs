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
fn decode_op_test() {
    let mut code = vec![
        0x00_u8, // nop
        0x01, 0b0001_0000, // stop ref(16)
        0x01, 0b1100_0000, 0xA, // stop const(10)
        0x01, 0b1000_0000, 0x0, // stop ref(0)
        0x02, 0b1100_0000, 0x1, // wait const(1)
        0x03, 0b1100_0000, 0x7, 0x0, // set b1 ret(0) const(7)
        0x03, 0b0001_0100, 0x0, 0x1, 0x1, // set b2 ref(1) ref(256)
        0x03, 0b1101_1000, 0x1, 0x1, 0x0, 0x0, 0x5, // set b2 ret(5) const(257)
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
    assert_eq!(set, Op::Set(RefRet::Return(0), Value::Const(7), 1));

    let set = decode_op(&mut code).unwrap();
    assert_eq!(set, Op::Set(RefRet::Ref(1), Value::Ref(256), 2));

    let set = decode_op(&mut code).unwrap();
    assert_eq!(set, Op::Set(RefRet::Return(5), Value::Const(257), 2));

    assert!(code.next().is_none());
}

#[test]
fn decode_op_test2() {
    let mut code = vec![
        0x00_u8, // nop
        0x04, 0b0000_0000, 0x7, 0x0, // add b1 0 ref(7)
        0x04, 0b1101_0000, 0x4, 0x7, 0x0, // add b2 0 const(7) 4
        0x04, 0b1101_0000, 0b1000_0000, 0x4, 0x7, 0x0, // add b2 0 const(7) 4
        0x05, 0b0010_0100, 0x7, 0x0, 0x1, // sub b4 1 ref(7)
        0x05, 0b0111_0100, 0x7, 0x0, 0x1, // sub b8 1 const(7)
    ].into_iter();

    let nop = decode_op(&mut code).unwrap();
    assert_eq!(nop, Op::Nop);

    let add = decode_op(&mut code).unwrap();
    assert_eq!(add, Op::Add(Ref(0), Value::Ref(7), None, 1));

    let add = decode_op(&mut code).unwrap();
    assert_eq!(add, Op::Add(Ref(0), Value::Const(7), Some(Ref(4)), 2));

    let add = decode_op(&mut code).unwrap();
    assert_eq!(add, Op::Add(Ref(0), Value::Const(7), Some(Ref(4)), 2));

    let sub = decode_op(&mut code).unwrap();
    assert_eq!(sub, Op::Sub(Ref(1), Value::Ref(7), None, 4));

    let sub = decode_op(&mut code).unwrap();
    assert_eq!(sub, Op::Sub(Ref(1), Value::Const(7), None, 8));

    assert!(code.next().is_none());
}

#[test]
fn decode_op_err() {
    let mut code = vec![0x01].into_iter();
    assert_eq!(decode_op(&mut code), Err(OpDecodeError::UnexpectedInputEnd));

    let mut code = vec![0x03, 0b1101_0000].into_iter();
    assert_eq!(decode_op(&mut code), Err(OpDecodeError::UnexpectedInputEnd));

    let mut code = vec![0xFF].into_iter();
    assert_eq!(decode_op(&mut code), Err(OpDecodeError::UnknownOpCode));
}
