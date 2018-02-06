#[macro_use]
extern crate rasm;

use rasm::x64::Assembler;
use rasm::defs::{Instruction, Operand, Reg};

type JitFn = extern "win64" fn(i64, i64) -> i64;

fn main() {
    let mut asm = Assembler::new();
    asm.push_instruction(Instruction::Mov(Operand::Reg(Reg::RAX), Operand::Reg(Reg::RCX)));
    asm.push_instruction(Instruction::Mul(Operand::Reg(Reg::RAX), Operand::Reg(Reg::RDX)));
    asm.push_instruction(Instruction::Ret);
    let ops = asm.commit().unwrap();
    println!("{}", asm.buffer_fmt());
    let fun = callable!(ops, JitFn);
    println!("{}", fun(2, 3));
}
