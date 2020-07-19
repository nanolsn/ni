/// No operation.
pub const NOP: u8 = 0x00;

/// End.
pub const END: u8 = 0x01;

/// Sleep.
pub const SLP: u8 = 0x02;

/// Set.
pub const SET: u8 = 0x03;

/// Convert.
pub const CNV: u8 = 0x04;

/// Addition.
pub const ADD: u8 = 0x05;

/// Subtraction.
pub const SUB: u8 = 0x06;

/// Multiplication.
pub const MUL: u8 = 0x07;

/// Division.
pub const DIV: u8 = 0x08;

/// Modulo.
pub const MOD: u8 = 0x09;

/// Shift left.
pub const SHL: u8 = 0x0A;

/// Shift right.
pub const SHR: u8 = 0x0B;

/// Bitwise and.
pub const AND: u8 = 0x0C;

/// Bitwise or.
pub const OR: u8 = 0x0D;

/// Bitwise xor.
pub const XOR: u8 = 0x0E;

/// Bitwise not.
pub const NOT: u8 = 0x0F;

/// Negate.
pub const NEG: u8 = 0x10;

/// Increment.
pub const INC: u8 = 0x11;

/// Decrement.
pub const DEC: u8 = 0x12;

/// Go to.
pub const GO: u8 = 0x13;

/// If true.
pub const IFT: u8 = 0x14;

/// If false.
pub const IFF: u8 = 0x15;

/// If equals.
pub const IFE: u8 = 0x16;

/// If less.
pub const IFL: u8 = 0x17;

/// If greater.
pub const IFG: u8 = 0x18;

/// If not equals.
pub const INE: u8 = 0x19;

/// If not less.
pub const INL: u8 = 0x1A;

/// If not greater.
pub const ING: u8 = 0x1B;

/// If bitwise and.
pub const IFA: u8 = 0x1C;

/// If bitwise or.
pub const IFO: u8 = 0x1D;

/// If bitwise xor.
pub const IFX: u8 = 0x1E;

/// If not bitwise and.
pub const INA: u8 = 0x1F;

/// If not bitwise or.
pub const INO: u8 = 0x20;

/// If not bitwise xor.
pub const INX: u8 = 0x21;

/// Append stackframe.
pub const APP: u8 = 0x22;

/// Function parameter.
pub const PAR: u8 = 0x23;

/// Call function.
pub const CLF: u8 = 0x24;

/// Return from function.
pub const RET: u8 = 0x25;

/// Input.
pub const IN: u8 = 0x26;

/// Output.
pub const OUT: u8 = 0x27;

/// Flush.
pub const FLS: u8 = 0x28;

/// Open file.
pub const OPN: u8 = 0x29;

/// Close file.
pub const CLS: u8 = 0x2A;

/// Set file descriptor.
pub const SFD: u8 = 0x2B;

/// Get file descriptor.
pub const GFD: u8 = 0x2C;
