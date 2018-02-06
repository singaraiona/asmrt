#![feature(test)]
extern crate test;
#[macro_use]
extern crate rasm;

use rasm::x64::Assembler;
use rasm::defs::{Instruction, Operand, Reg};

type JitFn = extern "win64" fn(i64, i64) -> i64;

macro_rules! test {
    ($fun:tt; $($ins:expr),+; $($prms:expr),+; $res:expr) => {
        #[test]
        fn $fun() {
            let mut asm = Assembler::new();
            $(asm.push_instruction($ins);)+
            asm.push_instruction(Instruction::Ret);
            let ops = asm.commit().unwrap();
            println!("{}", asm.buffer_fmt());
            let fun = callable!(ops, JitFn);
            let ret = fun($($prms),+);
            assert_eq!(ret, $res);
        }
    }
}
// Tests
test!(add_reg_reg;
    Instruction::Mov(Operand::Reg(Reg::RAX), Operand::Reg(Reg::RCX)),
    Instruction::Add(Operand::Reg(Reg::RAX), Operand::Reg(Reg::RDX));
    2, 3;
    5
);

test!(sub_reg_reg;
    Instruction::Mov(Operand::Reg(Reg::RAX), Operand::Reg(Reg::RCX)),
    Instruction::Sub(Operand::Reg(Reg::RAX), Operand::Reg(Reg::RDX));
    2, 3;
    -1
);

test!(mul_reg_reg;
    Instruction::Mov(Operand::Reg(Reg::RAX), Operand::Reg(Reg::RCX)),
    Instruction::Mul(Operand::Reg(Reg::RAX), Operand::Reg(Reg::RDX));
    2, 3;
    6
);

test!(mul_push_pop;
    Instruction::Push(Operand::Reg(Reg::RCX)),
    Instruction::Push(Operand::Reg(Reg::RDX)),
    Instruction::Pop(Operand::Reg(Reg::RAX)),
    Instruction::Pop(Operand::Reg(Reg::R9)),
    Instruction::Mul(Operand::Reg(Reg::RAX), Operand::Reg(Reg::R9));
    2, 3;
    6
);
