
const REX:      u8 = 0b01001000;
const MODREGRM: u8 = 0b11000000;

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

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Opcode {
    Add = 0x01,
    Sub = 0x29,
    Mov = 0x89,
    Ret = 0xc3,
}

#[derive(Debug, Clone, Copy)]
pub struct Operand(Option<Reg>);

impl Operand {
    pub fn reg(r: Reg) -> Self { Operand(Some(r)) }
    pub fn domain(&self) -> u8 { if let Some(o) = self.0 { o as u8 / 8 } else { 0 } }
    pub fn offset(&self) -> u8 { if let Some(o) = self.0 { o as u8 % 8 } else { 0 } }
}

pub enum Instruction {
    Nullary(Opcode),
    Unary(Opcode, Operand),
    Binary(Opcode, Operand, Operand),
}

impl Instruction {
    pub fn serialize(&self, buf: &mut Vec<u8>) {
        match *self {
            Instruction::Nullary(opcode) => {
                buf.push(opcode as u8);
            }
            Instruction::Binary(opcode, op1, op2) => {
                buf.push(REX | op1.domain() << 2 | op2.domain());
                buf.push(opcode as u8);
                buf.push(MODREGRM | op2.offset() << 3 | op1.offset());
            }
            _ => unimplemented!(),
        }
    }
}
