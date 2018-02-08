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
    Byte(i8),
    Word(i16),
    Dword(i32),
    Qword(i64),
    Float(f64),
    Lbl(&'static str),
    Mem,
}
//
pub enum Instruction {
    Add(Operand, Operand), // Add r64 to r/m64/xmm.
    Sub(Operand, Operand), // Subtract r/m64/xmm from r64.
    Mul(Operand, Operand), // Quadword register ← r/m64∗ immediate doubleword.
    Mov(Operand, Operand), // Move r/m64/imm to r64.
    Cmp(Operand, Operand), // Compare r/m64with r64.
    Jmp(Operand),          // Jump near, relative, RIP = RIP + 32-bit displacement sign extended to 64-bits
    Ja(Operand),           // Jump near if above (CF=0 and ZF=0)
    Jae(Operand),          // Jump near if above or equal (CF=0)
    Jb(Operand),           // Jump near if below (CF=1)
    Jbe(Operand),          // Jump near if below or equal (CF=1 or ZF=1)
    Jc(Operand),           // Jump near if carry (CF=1)
    Je(Operand),           // Jump near if equal (ZF=1)
    Jg(Operand),           // Jump near if greater (ZF=0 and SF=OF)
    Jge(Operand),          // Jump near if greater or equal (SF=OF)
    Jl(Operand),           // Jump near if less (SF<>OF)
    Jle(Operand),          // Jump near if less or equal (ZF=1 or SF<>OF)
    Jna(Operand),          // Jump near if not above (CF=1 or ZF=1)
    Jnae(Operand),         // Jump near if not above or equal (CF=1)
    Jnb(Operand),          // Jump near if not below (CF=0)
    Jnbe(Operand),         // Jump near if not below or equal (CF=0 and ZF=0)
    Jnc(Operand),          // Jump near if not carry (CF=0)
    Jne(Operand),          // Jump near if not equal (ZF=0)
    Jng(Operand),          // Jump near if not greater (ZF=1 or SF<>OF)
    Jnge(Operand),         // Jump near if not greater or equal (SF<>OF)
    Jnl(Operand),          // Jump near if not less (SF=OF)
    Jnle(Operand),         // Jump near if not less or equal (ZF=0 and SF=OF)
    Jno(Operand),          // Jump near if not overflow (OF=0)
    Jnp(Operand),          // Jump near if not parity (PF=0)
    Jns(Operand),          // Jump near if not sign (SF=0)
    Jnz(Operand),          // Jump near if not zero (ZF=0)
    Jo(Operand),           // Jump near if overflow (OF=1)
    Jp(Operand),           // Jump near if parity (PF=1)
    Jpe(Operand),          // Jump near if parity even (PF=1)
    Jpo(Operand),          // Jump near if parity odd (PF=0)
    Js(Operand),           // Jump near if sign (SF=1)
    Jz(Operand),           // Jump near if 0 (ZF=1)
    Push(Operand),         // Push r/m64.
    Pop(Operand),          // Pop top of stack into r64; increment stack pointer. Cannot encode 32-bit operand size.
    SetLbl(&'static str),  // Set label
    Call(Operand),         // Call near, absolute indirect, address given in r/m64.
    Ret,                   // Near return to calling procedure.
}
//
