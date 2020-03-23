use crate::decoder::*;
use crate::operation::*;

fn decode(bytes: Vec<u8>) -> Result<Op, OpDecodeError> { decode_op(&mut bytes.into_iter()) }

#[test]
fn decode_op_test() {
    let code = vec![0x00]; // nop
    let nop = decode(code).unwrap();
    assert_eq!(nop, Op::Nop);

    let code = vec![0x01, 0b0001_0000]; // stop ref(16)
    let stop = decode(code).unwrap();
    assert_eq!(stop, Op::Stop(Value::Ref(16)));

    let code = vec![0x01, 0b1100_0000, 0xA]; // stop const(10)
    let stop = decode(code).unwrap();
    assert_eq!(stop, Op::Stop(Value::Const(10)));

    let code = vec![0x01, 0b1000_0000, 0x0]; // stop ref(0)
    let stop = decode(code).unwrap();
    assert_eq!(stop, Op::Stop(Value::Ref(0)));

    let code = vec![0x02, 0b1100_0000, 0x1]; // wait const(1)
    let wait = decode(code).unwrap();
    assert_eq!(wait, Op::Wait(Value::Const(1)));

    let code = vec![0x03, 0b1100_0000, 0x7, 0x0]; // set b1 ret(0) const(7)
    let set = decode(code).unwrap();
    assert_eq!(set, Op::Set(RefRet::Return(0), Value::Const(7), OpSize::B1));

    let code = vec![0x03, 0b0001_0100, 0x0, 0x1, 0x1]; // set b2 ref(1) ref(256)
    let set = decode(code).unwrap();
    assert_eq!(set, Op::Set(RefRet::Ref(1), Value::Ref(256), OpSize::B2));

    let code = vec![0x03, 0b1101_1000, 0x1, 0x1, 0x0, 0x0, 0x5]; // set b2 ret(5) const(257)
    let set = decode(code).unwrap();
    assert_eq!(set, Op::Set(RefRet::Return(5), Value::Const(257), OpSize::B2));
}

#[test]
fn decode_op_test2() {
    let code = vec![0x04, 0b0000_0000, 0x7, 0x0]; // add b1 0 ref(7)
    let add = decode(code).unwrap();
    assert_eq!(add, Op::Add(Ref(0), Value::Ref(7), None, OpSize::B1));

    let code = vec![0x04, 0b1101_0000, 0x4, 0x7, 0x0]; // add b2 0 const(7) 4
    let add = decode(code).unwrap();
    assert_eq!(add, Op::Add(Ref(0), Value::Const(7), Some(Ref(4)), OpSize::B2));

    let code = vec![0x04, 0b1101_0000, 0b1000_0000, 0x4, 0x7, 0x0]; // add b2 0 const(7) 4
    let add = decode(code).unwrap();
    assert_eq!(add, Op::Add(Ref(0), Value::Const(7), Some(Ref(4)), OpSize::B2));

    let code = vec![0x05, 0b0010_0100, 0x7, 0x0, 0x1]; // sub b4 1 ref(7)
    let sub = decode(code).unwrap();
    assert_eq!(sub, Op::Sub(Ref(1), Value::Ref(7), None, OpSize::B4));

    let code = vec![0x05, 0b0111_0100, 0x7, 0x0, 0x1]; // sub b8 1 const(7)
    let sub = decode(code).unwrap();
    assert_eq!(sub, Op::Sub(Ref(1), Value::Const(7), None, OpSize::B8));
}

#[test]
fn decode_op_err() {
    use OpDecodeError::*;

    let code = vec![0x01];
    assert_eq!(decode(code), Err(UnexpectedEnd(OpExpected::Spec(0))));

    let code = vec![0x04, 0b1101_0000];
    assert_eq!(decode(code), Err(UnexpectedEnd(OpExpected::Spec(1))));

    let code = vec![0x03, 0b1101_0000];
    assert_eq!(decode(code), Err(UnexpectedEnd(OpExpected::Operand(1))));

    let code = vec![0x03, 0b1101_0000, 0x0];
    assert_eq!(decode(code), Err(UnexpectedEnd(OpExpected::Operand(0))));

    let code = vec![0xFF];
    assert_eq!(decode(code), Err(UnknownOpCode));
}
