// REX
pub const REX:   u8 = 0b01001000;
// ModRegR/M
pub const MOD_ADDR_REG:       u8 = 0xc0;
pub const MOD_ADDR_WO_OFFSET: u8 = 0x00;
pub const MOD_ADDR_W_OFFSET:  u8 = 0x80;
//
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Ireg {
   RAX, // 0.000
   RCX, // 0.001
   RDX, // 0.010
   RBX, // 0.011
   RSP, // 0.100
   RBP, // 0.101
   RSI, // 0.110
   RDI, // 0.111
   R8 , // 1.000
   R9 , // 1.001
   R10, // 1.010
   R11, // 1.011
   R12, // 1.100
   R13, // 1.101
   R14, // 1.110
   R15, // 1.111
}
//
impl Ireg {
    pub fn rex(&self) -> u8 { *self as u8 / 8 }
    pub fn reg(&self) -> u8 { *self as u8 % 8 }
}
//
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Freg {
    XMM0,
    XMM1,
    XMM2,
    XMM3,
    XMM4,
    XMM5,
    XMM6,
    XMM7,
}
//
impl Freg {
    pub fn reg(&self) -> u8 { *self as u8 }
}
//
#[derive(Debug, Clone, Copy)]
pub enum Operand {
    Ireg(Ireg),
    Freg(Freg),
    Iimm(i64),
    Fimm(f64),
    Label(&'static str),
    Mem,
}
//
pub enum Instruction {
    Add(Operand, Operand),
    Sub(Operand, Operand),
    Mul(Operand, Operand),
    Mov(Operand, Operand),
    Cmp(Operand, Operand),
    Jmp(Operand),
    Push(Operand),
    Pop(Operand),
    SetLabel(&'static str),
    Ret,
}
//
