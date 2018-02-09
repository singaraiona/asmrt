use memmap::{Mmap, Protection};
use std::io::Cursor;
use std::io::prelude::*;
use std::mem;
use ops::*;
use Error;
use fnv::FnvHashMap;
//
const MMAP_INIT_SIZE: usize = 1024 * 256;
//
macro_rules! or          { ($e:expr)                                 => {
                           { let _ = $e.or_else(|_| Err(Error::Serialize))?;                                      }}}
macro_rules! push_prefix { ($w:expr, $op1:expr, $op2:expr)           => { or!($w.write(&[REX | $op1 << 2 | $op2])) }}
macro_rules! push_opcode { ($w:expr, $($op:expr),+)                  => { or!($w.write(&[$($op),+]))               }}
macro_rules! push_modreg { ($w:expr, $md:expr, $op1:expr, $op2:expr) => { or!($w.write(&[$md | $op1 << 3 | $op2])) }}
macro_rules! as_bytes    { ($t:tt,   $v:expr)                        => {
                            unsafe { &mem::transmute::<$t,[u8;mem::size_of::<$t>()]>($v) }                         }}
macro_rules! push_imm8   { ($w:expr, $im:expr)                       => { or!($w.write(&[$im]))                    }}
macro_rules! push_imm16  { ($w:expr, $im:expr)                       => { or!($w.write(as_bytes!(i16, $im)))       }}
macro_rules! push_imm32  { ($w:expr, $im:expr)                       => { or!($w.write(as_bytes!(i32, $im)))       }}
macro_rules! push_imm64  { ($w:expr, $im:expr)                       => { or!($w.write(as_bytes!(i64, $im)))       }}
//
macro_rules! op {
    ($w:expr, $($b:expr),+) => {
        or!($w.write(&[$($b),+]))
    };
    // OP
    ($w:expr, $($oc:expr),+, /r $op1:tt $op2:tt $($e:expr),*) => {
        or!($w.write(&[$($oc),+, MOD_ADDR_REG | ($op1 as u8 % 8) << 3 | ($op2 as u8 % 8), $($e),*]))
    };
    // OP + pi
    ($w:expr, $oc:expr, +$pi:tt, /r $op1:tt $op2:tt $($e:expr),*) => {
        or!($w.write(&[$($oc),+, MOD_ADDR_REG | ($op1 as u8 % 8) << 3 | ($op2 as u8 % 8), $($e),*]))
    };
    // REX + OP
    ($w:expr, $p:expr, $($oc:expr),+, /r $op1:tt $op2:tt $($e:expr),*) => {
        or!($w.write($p | ($op1 as u8 / 8) << 2 | ($op2 as u8 / 8)));
        op!($w, $($oc),+, /r $op1 $op2 $($e),*)
    };
    // REX + OP + pi
    ($w:expr, $p:expr, $oc:expr, +$pi:tt, /r $op1:tt $op2:tt $($e:expr),*) => {
        or!($w.write($p | ($op1 as u8 / 8) << 2 | ($op2 as u8 / 8)));
        op!($w, $oc | $pi as u8, /r $op1 $op2 $($e),*)
    }
}
//
pub struct Code(Mmap);

impl Code {
    pub fn ptr(&self) -> *const u8 { self.0.ptr() }
}
//
pub struct Assembler {
    labels:   FnvHashMap<&'static str, u64>, // lables str to offset in buffer mapping
    mentions: Vec<(&'static str, u64)>,      // set of labels mentioned inside buffer
    buffer:   Cursor<Vec<u8>>,               // executable code
}
//
impl Assembler {
    pub fn new() -> Self {
        Assembler {
            labels:   FnvHashMap::with_capacity_and_hasher(16, Default::default()),
            mentions: vec![],
            buffer:   Cursor::new(vec![])
        }
    }

    pub fn buffer_fmt(&self) -> String {
        format!("[{}]", self.buffer.get_ref().iter().map(|b| format!("0x{:02x}", b)).collect::<Vec<_>>().join(" "))
    }

    pub fn push_instruction(&mut self, i: Instruction) -> Result<(), Error> { self.serialize_instruction(i) }

    pub fn commit(&mut self) -> Result<Code, Error> {
        self.resolve_labels()?;
        //println!("{}", self.buffer_fmt());
        //return Err(Error::UnknownLabel);
        let mut mm = Mmap::anonymous(MMAP_INIT_SIZE, Protection::ReadWrite).or_else(|_| Err(Error::MmapCreate))?;
        {
            let buf = unsafe { mm.as_mut_slice() };
            for (i, v) in self.buffer.get_ref().iter().enumerate() { buf[i] = *v; }
        }
        mm.set_protection(Protection::ReadExecute).or_else(|_| Err(Error::MmapSetMode))?;
        Ok(Code(mm))
    }

    fn resolve_labels(&mut self) -> Result<(), Error> {
       for lbl in &self.mentions {
           let pos = self.labels.get(lbl.0).ok_or_else(|| Error::UnknownLabel)?;
           self.buffer.set_position(lbl.1);
           push_imm32!(&mut self.buffer, *pos as i32 - lbl.1 as i32 - mem::size_of::<i32>() as i32);
       }
       Ok(())
    }

    fn jump_near(&mut self, to: &'static str) -> Result<(), Error> {
        let offset = self.buffer.position();
        push_imm32!(&mut self.buffer, 0);
        self.mentions.push((to, offset));
        Ok(())
    }

    fn serialize_instruction(&mut self, instr: Instruction) -> Result<(), Error> {
        use ops::Instruction::*;
        use ops::Operand::*;
        match instr {
            Ret => push_opcode!(&mut self.buffer, 0xc3),
            Add(op1, op2) => {
                match (op1, op2) {
                    (Ireg(r1), Ireg(r2)) => op!(&mut self.buffer, REX, 0x01, /r r2 r1),
                    (Freg(r1), Freg(r2)) => op!(&mut self.buffer, 0xf2, 0x0f, 0x58, /r r1 r2),
                    (Ireg(r1), Byte(b2)) => {
                        op!(&mut self.buffer, REX, 0x83, /r 0 r1 b2 as u8);
                        //push_prefix!(&mut self.buffer, 0, r1.rex());
                        //push_opcode!(&mut self.buffer, 0x83);
                        //push_modreg!(&mut self.buffer, MOD_ADDR_REG, 0, r1.reg());
                        //push_imm8!(&mut self.buffer, b2 as u8);
                    }
                    _ => unimplemented!(),
                }
            }
            Sub(op1, op2) => {
                match (op1, op2) {
                    (Ireg(r1), Ireg(r2)) => {
                        push_prefix!(&mut self.buffer, r2.rex(), r1.rex());
                        push_opcode!(&mut self.buffer, 0x29);
                        push_modreg!(&mut self.buffer, MOD_ADDR_REG, r2.reg(), r1.reg());
                    }
                    (Ireg(r1), Byte(b2)) => {
                        push_prefix!(&mut self.buffer, 0, r1.rex());
                        push_opcode!(&mut self.buffer, 0x83);
                        push_modreg!(&mut self.buffer, MOD_ADDR_REG, 0x05, r1.reg());
                        push_imm8!(&mut self.buffer, b2 as u8);
                    }
                    _ => unimplemented!(),
                }
            }
            Mul(op1, op2) => {
                match (op1, op2) {
                    (Ireg(r1), Ireg(r2)) => {
                        push_prefix!(&mut self.buffer, r1.rex(), r2.rex());
                        push_opcode!(&mut self.buffer, 0x0f, 0xaf);
                        push_modreg!(&mut self.buffer, MOD_ADDR_REG, r1.reg(), r2.reg());
                    }
                    _ => unimplemented!(),
                }
            }
            Mov(op1, op2) => {
                match (op1, op2) {
                    (Ireg(r1), Ireg(r2)) => op!(&mut self.buffer, REX, 0x89, /r r2 r1),
                    (Freg(r1), Freg(r2)) => op!(&mut self.buffer, REX, 0x66, 0x0f, 0x6f, /r r1 r2),
                    (Ireg(r1), Qword(i2)) => {
                        op!(&mut self.buffer, REX, 0xb8 + r1, i2);
                        //push_prefix!(&mut self.buffer, 0, 0);
                        //push_opcode!(&mut self.buffer, 0xb8 | r1.reg());
                        //push_imm64!(&mut self.buffer, i2);
                    }
                    _ => unimplemented!(),
                }
            }
            Cmp(op1, op2) => {
                match (op1, op2) {
                    (Ireg(r1), Ireg(r2)) => {
                        op!(&mut self.buffer, REX, 0x3b, /r r1 r2);
                        //push_prefix!(&mut self.buffer, r1.rex(), r2.rex());
                        //push_opcode!(&mut self.buffer, 0x3b);
                        //push_modreg!(&mut self.buffer, MOD_ADDR_REG, r1.reg(), r2.reg());
                    }
                    _ => unimplemented!(),
                }
            }
            Push(op1) => {
                match op1 {
                    Ireg(r1) => {
                        op!(&mut self.buffer, REX, 0x50 +r1.reg());
                        //if r1.rex() == 1 { push_prefix!(&mut self.buffer, 0, 1); }
                        //push_opcode!(&mut self.buffer, 0x50 | r1.reg());
                    }
                    _ => unimplemented!(),
                }
            }
            Pop(op1) => {
                match op1 {
                    Ireg(r1) => {
                        if r1.rex() == 1 { push_prefix!(&mut self.buffer, 0, 1); }
                        push_opcode!(&mut self.buffer, 0x58 | r1.reg());
                    }
                    _ => unimplemented!(),
                }
            }
            SetLbl(l) => {
                let offset = self.buffer.position();
                self.labels.insert(l, offset);
            }
            Jmp(op1) => {
                match op1 {
                    Lbl(l) => {
                        push_opcode!(&mut self.buffer, 0xe9);
                        self.jump_near(l)?;
                    }
                    _ => unimplemented!(),
                }
            }
            Jne(op1) => {
                match op1 {
                    Lbl(l) => {
                        push_opcode!(&mut self.buffer, 0x0f, 0x85);
                        self.jump_near(l)?;
                    }
                    _ => unimplemented!(),
                }
            }
            Call(op1) => {
                match op1 {
                    Ireg(r1) => {
                        push_opcode!(&mut self.buffer, 0xff);
                        push_modreg!(&mut self.buffer, MOD_ADDR_REG, 0x02, r1.reg());
                    }
                    _ => unimplemented!(),
                }
            }
            _ => unimplemented!(),
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! callable { ($c:expr, $t:ty) => {
    {
        let _fun: $t = unsafe { ::std::mem::transmute($c.ptr()) };
        _fun
    }
}}
