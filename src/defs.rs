use std::io::prelude::*;
use std::io::Cursor;
use std::mem;

// REX
const REX:   u8 = 0b01001000;
// ModRegR/M
const MOD_ADDR_REG:       u8 = 0xc0;
const MOD_ADDR_WO_OFFSET: u8 = 0x00;
const MOD_ADDR_W_OFFSET:  u8 = 0x80;

#[derive(Debug, Clone, Copy)]
pub enum Error {
    Serialize,
}
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

impl Ireg {
    pub fn rex(&self) -> u8 { *self as u8 / 8 }
    pub fn reg(&self) -> u8 { *self as u8 % 8 }
}

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

impl Freg {
    pub fn reg(&self) -> u8 { *self as u8 }
}

#[derive(Debug, Clone, Copy)]
pub enum Operand {
    Ireg(Ireg),
    Freg(Freg),
    Iimm(i64),
    Fimm(f64),
    Mem,
}

pub enum Instruction {
    Add(Operand, Operand),
    Sub(Operand, Operand),
    Mul(Operand, Operand),
    Mov(Operand, Operand),
    Cmp(Operand, Operand),
    Push(Operand),
    Pop(Operand),
    Ret,
}
//
macro_rules! or          { ($e:expr)                                 => { $e.or_else(|_| Err(Error::Serialize))?   }}
macro_rules! push_prefix { ($w:expr, $op1:expr, $op2:expr)           => { or!($w.write(&[REX | $op1 << 2 | $op2])) }}
macro_rules! push_opcode { ($w:expr, $($op:expr),+)                  => { or!($w.write(&[$($op),+]))               }}
macro_rules! push_modreg { ($w:expr, $md:expr, $op1:expr, $op2:expr) => { or!($w.write(&[$md | $op1 << 3 | $op2])) }}
macro_rules! push_immi64 { ($w:expr, $im:expr)                       => {
                            unsafe { or!($w.write(&mem::transmute::<i64, [u8;8]>($im))) }}}

impl Instruction {
    pub fn serialize<W: Write + Seek>(&self, writer: &mut W) -> Result<(), Error> {
        use self::Instruction::*;
        use self::Operand::*;
        match *self {
            Ret => {
                push_opcode!(writer, 0xc3);
            },
            Add(op1, op2) => {
                match (op1, op2) {
                    (Ireg(r1), Ireg(r2)) => {
                        push_prefix!(writer, r2.rex(), r1.rex());
                        push_opcode!(writer, 0x01);
                        push_modreg!(writer, MOD_ADDR_REG, r2.reg(), r1.reg());
                    }
                    // ADDSD
                    (Freg(r1), Freg(r2)) => {
                        push_opcode!(writer, 0xf2, 0x0f, 0x58);
                        push_modreg!(writer, MOD_ADDR_REG, r1.reg(), r2.reg());
                    }
                    _ => unimplemented!(),
                }
            }
            Sub(op1, op2) => {
                match (op1, op2) {
                    (Ireg(r1), Ireg(r2)) => {
                        push_prefix!(writer, r2.rex(), r1.rex());
                        push_opcode!(writer, 0x29);
                        push_modreg!(writer, MOD_ADDR_REG, r2.reg(), r1.reg());
                    }
                    _ => unimplemented!(),
                }
            }
            Mul(op1, op2) => {
                match (op1, op2) {
                    (Ireg(r1), Ireg(r2)) => {
                        push_prefix!(writer, r1.rex(), r2.rex());
                        push_opcode!(writer, 0x0f, 0xaf);
                        push_modreg!(writer, MOD_ADDR_REG, r1.reg(), r2.reg());
                    }
                    _ => unimplemented!(),
                }
            }
            Mov(op1, op2) => {
                match (op1, op2) {
                    // MOVQ
                    (Ireg(r1), Ireg(r2)) => {
                        push_prefix!(writer, r2.rex(), r1.rex());
                        push_opcode!(writer, 0x89);
                        push_modreg!(writer, MOD_ADDR_REG, r2.reg(), r1.reg());
                    }
                    // MOVDQA
                    (Freg(r1), Freg(r2)) => {
                        push_opcode!(writer, 0x66, 0x0f, 0x6f);
                        push_modreg!(writer, MOD_ADDR_REG, r1.reg(), r2.reg());
                    }
                    (Ireg(r1), Iimm(i2)) => {
                        push_prefix!(writer, 0, 0);
                        push_opcode!(writer, 0xb8);
                        push_immi64!(writer, i2);
                    }
                    _ => unimplemented!(),
                }
            }
            Cmp(op1, op2) => {
                match (op1, op2) {
                    (Ireg(r1), Ireg(r2)) => {
                        push_prefix!(writer, r2.rex(), r1.rex());
                        push_opcode!(writer, 0x39);
                        push_modreg!(writer, MOD_ADDR_REG, r2.reg(), r1.reg());
                    }
                    //// MOVDQA
                    //(Freg(r1), Freg(r2)) => {
                        //push_opcode!(writer, 0x66, 0x0f, 0x6f);
                        //push_modreg!(writer, MOD_ADDR_REG, r1.reg(), r2.reg());
                    //}
                    _ => unimplemented!(),
                }
            }
            Push(op1) => {
                match op1 {
                    Ireg(r1) => {
                        if r1.rex() == 1 { push_prefix!(writer, 0, 1); }
                        push_opcode!(writer, 0x50 | r1.reg());
                    }
                    _ => unimplemented!(),
                }
            }
            Pop(op1) => {
                match op1 {
                    Ireg(r1) => {
                        if r1.rex() == 1 { push_prefix!(writer, 0, 1); }
                        push_opcode!(writer, 0x58 | r1.reg());
                    }
                    _ => unimplemented!(),
                }
            }

            _ => unimplemented!(),
        }
        Ok(())
    }
}

