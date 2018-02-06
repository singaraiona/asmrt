
const REX:       u8 = 0b01000000;
const LONG_MODE: u8 = 1 << 3;
const MODREGRM:  u8 = 0b11000000;

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Reg {
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

impl Reg {
    pub fn domain(&self) -> u8 { *self as u8 / 8 }
    pub fn offset(&self) -> u8 { *self as u8 % 8 }
}

#[derive(Debug, Clone, Copy)]
pub enum Operand {
    Reg(Reg),
    Mem,
}

pub enum Instruction {
    Add(Operand, Operand),
    Sub(Operand, Operand),
    Mul(Operand, Operand),
    Mov(Operand, Operand),
    Push(Operand),
    Pop(Operand),
    Ret,
}

impl Instruction {
    pub fn serialize(&self, buf: &mut Vec<u8>) {
        use self::Instruction::*;
        use self::Operand::*;
        match *self {
            Ret => buf.push(0xc3),
            Add(op1, op2) => {
                match (op1, op2) {
                    (Reg(r1), Reg(r2)) => {
                        buf.push(REX | LONG_MODE | r2.domain() << 2 | r1.domain());
                        buf.push(0x01);
                        buf.push(MODREGRM | r2.offset() << 3 | r1.offset());
                    }
                    _ => unimplemented!(),
                }
            }
            Sub(op1, op2) => {
                match (op1, op2) {
                    (Reg(r1), Reg(r2)) => {
                        buf.push(REX | LONG_MODE | r2.domain() << 2 | r1.domain());
                        buf.push(0x29);
                        buf.push(MODREGRM | r2.offset() << 3 | r1.offset());
                    }
                    _ => unimplemented!(),
                }
            }
            Mul(op1, op2) => {
                match (op1, op2) {
                    (Reg(r1), Reg(r2)) => {
                        buf.push(REX | LONG_MODE | r1.domain() << 2 | r2.domain());
                        buf.push(0x0f);
                        buf.push(0xaf);
                        buf.push(MODREGRM | r1.offset() << 3 | r2.offset());
                    }
                    _ => unimplemented!(),
                }
            }
            Mov(op1, op2) => {
                match (op1, op2) {
                    (Reg(r1), Reg(r2)) => {
                        buf.push(REX | r2.domain() << 2 | r1.domain());
                        buf.push(0x89);
                        buf.push(MODREGRM | r2.offset() << 3 | r1.offset());
                    }
                    _ => unimplemented!(),
                }
            }
            Push(op1) => {
                match op1 {
                    Reg(r1) => {
                        if r1.domain() == 1 { buf.push(REX | 1); }
                        buf.push(0x50 | r1.offset());
                    }
                    _ => unimplemented!(),
                }
            }
            Pop(op1) => {
                match op1 {
                    Reg(r1) => {
                        if r1.domain() == 1 { buf.push(REX | 1); }
                        buf.push(0x58 | r1.offset());
                    }
                    _ => unimplemented!(),
                }
            }
            _ => unimplemented!(),
        }
    }
}
