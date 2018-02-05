#[macro_use]
extern crate rasm;

use rasm::x64::Assembler;
use rasm::defs::{Instruction, Operand, Opcode, Reg};

type JitFn = extern "win64" fn(i64, i64) -> i64;

fn main() {
    let mut asm = Assembler::new();
    asm.push_instruction(Instruction::Binary(Opcode::Mov, Operand::reg(Reg::RAX), Operand::reg(Reg::RCX)));
    asm.push_instruction(Instruction::Binary(Opcode::Add, Operand::reg(Reg::RAX), Operand::reg(Reg::RDX)));
    asm.push_instruction(Instruction::Nullary(Opcode::Ret));
    let ops = asm.commit().unwrap();
    println!("{}", asm.content_fmt());
    let fun = callable!(ops, JitFn);
    println!("{}", fun(55, 50));
}
