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
macro_rules! bs { ($t:tt, $v:expr) => { unsafe { &mem::transmute::<$t, [u8;mem::size_of::<$t>()]>($v) }}}
macro_rules! ib { ($i:expr)        => { &[$i as u8]  }} // 1 - byte
macro_rules! iw { ($i:expr)        => { bs!(i16, $i) }} // 2 - byte
macro_rules! id { ($i:expr)        => { bs!(i32, $i) }} // 4 - byte
macro_rules! io { ($i:expr)        => { bs!(i64, $i) }} // 8 - byte

// Indicates the use of a REX prefix that affects operand size or instruction semantics.
macro_rules! rex {
    ($($op:tt),+; $r1:expr)           => { &[REX | ($r1 as u8 / 8), $($op),+, MOD_ADDR_REG | ($r1 as u8 % 8)] };
    ($($op:tt),+; $r1:expr, $r2:expr) => { &[REX | ($r1 as u8 / 8) << 2 | ($r2 as u8 / 8), $($op),+, MOD_ADDR_REG |
                                          ($r1 as u8 % 8) << 3 | ($r2 as u8 % 8)] }
}
// Indicates that the ModR/M byte of the instruction contains a register operand and an r/m operand without REX
macro_rules! reg {
    ($($op:tt),+; $r1:expr)           => { &[$($op),+, MOD_ADDR_REG | ($r1 as u8 % 8)] };
    ($($op:tt),+; $r1:expr, $r2:expr) => { &[$($op),+, MOD_ADDR_REG | ($r1 as u8 % 8) << 3 | ($r2 as u8 % 8)] }
}
// Indicated the lower 3 bits of the opcode byte is used to encode the register operand without a modR/M byte
macro_rules! pcd { ($op:tt; $r1:expr) => { &[$op | ($r1 as u8 % 8)] }}
//
macro_rules! op { ($w:expr, $($e:expr),+) => {{ $( $w.write($e).or_else(|_| Err(Error::Serialize))?;)+;} }}
//
//macro_rules! op_dyad {
    //($w:expr, $op1:expr, $op2:expr, $r1:tt, $r2:tt, $($arm:expr),+) => {
        //match ($op1, $op2) => {
            //$(
               //() 
            //)+
            //_ => unimplemented!(),
        //}
    //}
//}
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
           op!(&mut self.buffer, id!(*pos as i32 - lbl.1 as i32 - mem::size_of::<i32>() as i32));
       }
       Ok(())
    }

    fn jump_near(&mut self, to: &'static str) -> Result<(), Error> {
        let offset = self.buffer.position();
        op!(&mut self.buffer, id!(0));
        self.mentions.push((to, offset));
        Ok(())
    }

    fn serialize_instruction(&mut self, instr: Instruction) -> Result<(), Error> {
        use ops::Instruction::*;
        use ops::Operand::*;
        match instr {
            Ret => op!(&mut self.buffer, ib!(0xc3)),
            Add(op1, op2) => {
                match (op1, op2) {
                    (Ireg(r1), Ireg(r2)) => op!(&mut self.buffer, rex!(0x01; r2, r1)),
                    (Freg(r1), Freg(r2)) => op!(&mut self.buffer, reg!(0xf2, 0x0f, 0x58; r1, r2)),
                    (Ireg(r1), Byte(b2)) => op!(&mut self.buffer, rex!(0x83; r1), ib!(b2)),
                    _ => unimplemented!(),
                }
            }
            Sub(op1, op2) => {
                match (op1, op2) {
                    (Ireg(r1), Ireg(r2)) => op!(&mut self.buffer, rex!(0x29; r2, r1)),
                    (Ireg(r1), Byte(b2)) => op!(&mut self.buffer, rex!(0x83; 0x05, r1), ib!(b2)),
                    _ => unimplemented!(),
                }
            }
            Mul(op1, op2) => {
                match (op1, op2) {
                    (Ireg(r1), Ireg(r2)) => op!(&mut self.buffer, rex!(0x0f, 0xaf; r1, r2)),
                    _ => unimplemented!(),
                }
            }
            Mov(op1, op2) => {
                match (op1, op2) {
                    (Ireg(r1), Ireg(r2))  => op!(&mut self.buffer, rex!(0x89; r2, r1)),
                    (Freg(r1), Freg(r2))  => op!(&mut self.buffer, rex!(0x66, 0x0f, 0x6f; r1, r2)),
                    (Ireg(r1), Qword(i2)) => op!(&mut self.buffer, &[REX, 0xb8 | r1.reg()], io!(i2)),
                    _ => unimplemented!(),
                }
            }
            Cmp(op1, op2) => {
                match (op1, op2) {
                    (Ireg(r1), Ireg(r2)) => op!(&mut self.buffer, rex!(0x3b; r1, r2)),
                    _ => unimplemented!(),
                }
            }
            Push(op1) => {
                match op1 {
                    Ireg(r1) => {
                        if r1.rex() == 1 { op!(&mut self.buffer, ib!(REX | 1), pcd!(0x50; r1)); }
                        else             { op!(&mut self.buffer, pcd!(0x50; r1)); }
                    },
                    _ => unimplemented!(),
                }
            }
            Pop(op1) => {
                match op1 {
                    Ireg(r1) => {
                        if r1.rex() == 1 { op!(&mut self.buffer, ib!(REX | 1), pcd!(0x58; r1)); }
                        else             { op!(&mut self.buffer, pcd!(0x58; r1)); }
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
                        op!(&mut self.buffer, ib!(0xe9));
                        self.jump_near(l)?;
                    }
                    _ => unimplemented!(),
                }
            }
            Jne(op1) => {
                match op1 {
                    Lbl(l) => {
                        op!(&mut self.buffer, &[0x0f, 0x85]);
                        self.jump_near(l)?;
                    }
                    _ => unimplemented!(),
                }
            }
            Call(op1) => {
                match op1 {
                    Ireg(r1) => op!(&mut self.buffer, reg!(0xff; 0x02, r1)),
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
