/// No operation.
pub const NOP: u8 = 0x00;

/// End.
pub const END: u8 = 0x01;

/// Sleep.
pub const SLP: u8 = 0x02;

/// Set.
pub const SET: u8 = 0x03;

/// Addition.
pub const ADD: u8 = 0x04;

/// Subtraction.
pub const SUB: u8 = 0x05;

/// Multiplication.
pub const MUL: u8 = 0x06;

/// Division.
pub const DIV: u8 = 0x07;

/// Modulo.
pub const MOD: u8 = 0x08;

/// Shift left.
pub const SHL: u8 = 0x09;

/// Shift right.
pub const SHR: u8 = 0x0A;

/// Bitwise and.
pub const AND: u8 = 0x0B;

/// Bitwise or.
pub const OR: u8 = 0x0C;

/// Bitwise xor.
pub const XOR: u8 = 0x0D;

/// Bitwise not.
pub const NOT: u8 = 0x0E;

/// Negate.
pub const NEG: u8 = 0x0F;

/// Increment.
pub const INC: u8 = 0x10;

/// Decrement.
pub const DEC: u8 = 0x11;

/// Go to.
pub const GO: u8 = 0x12;

/// If true.
pub const IFT: u8 = 0x13;

/// If false.
pub const IFF: u8 = 0x14;

/// If equals.
pub const IFE: u8 = 0x15;

/// If less.
pub const IFL: u8 = 0x16;

/// If greater.
pub const IFG: u8 = 0x17;

/// If not equals.
pub const INE: u8 = 0x18;

/// If not less.
pub const INL: u8 = 0x19;

/// If not greater.
pub const ING: u8 = 0x2A;

/// If bitwise and.
pub const IFA: u8 = 0x2B;

/// If bitwise or.
pub const IFO: u8 = 0x2C;

/// If bitwise xor.
pub const IFX: u8 = 0x2D;

/// If not bitwise and.
pub const INA: u8 = 0x2E;

/// If not bitwise or.
pub const INO: u8 = 0x2F;

/// If not bitwise xor.
pub const INX: u8 = 0x30;

/// Append stackframe.
pub const APP: u8 = 0x31;

/// Function parameter.
pub const PAR: u8 = 0x32;

/// Call function.
pub const CFN: u8 = 0x33;

/// Input.
pub const IN: u8 = 0x34;

/// Output.
pub const OUT: u8 = 0x35;
