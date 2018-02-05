use memmap::{Mmap, Protection};
use defs::*;
use Error;

const MMAP_INIT_SIZE: usize = 1024 * 256;

pub struct Ops(Mmap);

impl Ops {
    pub fn ptr(&self) -> *const u8 { self.0.ptr() }
}

pub struct Assembler {
    buffer: Vec<u8>,
}

impl Assembler {
    pub fn new() -> Self { Assembler { buffer: vec![] } }

    pub fn content_fmt(&self) -> String {
        format!("[{}]", self.buffer.iter().map(|b| format!("0x{:02x}", b)).collect::<Vec<_>>().join(" "))
    }

    pub fn push_instruction(&mut self, i: Instruction) { i.serialize(&mut self.buffer); }

    pub fn commit(&mut self) -> Result<Ops, Error> {
        let mut mm = Mmap::anonymous(MMAP_INIT_SIZE, Protection::ReadWrite).or_else(|_| Err(Error::MmapCreate))?;
        {
            let buf = unsafe { mm.as_mut_slice() };
            for (i, v) in self.buffer.iter().enumerate() { buf[i] = *v; }
        }
        mm.set_protection(Protection::ReadExecute).or_else(|_| Err(Error::MmapSetMode))?;
        Ok(Ops(mm))
    }
}

#[macro_export]
macro_rules! callable { ($c:expr, $t:ty) => {
    {
        let _fun: $t = unsafe { ::std::mem::transmute($c.ptr()) };
        _fun
    }
}}
