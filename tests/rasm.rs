#![feature(test)]
extern crate test;
#[macro_use]
extern crate rasm;

use rasm::x64::Assembler;
use rasm::defs::{Instruction, Operand, Ireg, Freg};

type MonadI64 = extern "win64" fn(i64)      -> i64;
type DyadI64  = extern "win64" fn(i64, i64) -> i64;
type DyadF64  = extern "win64" fn(f64, f64) -> f64;

macro_rules! test {
    ($fun:tt; $($ins:expr),+; $jf:ty; $($prms:expr),+; $res:expr) => {
        #[test]
        #[allow(unused)]
        fn $fun() {
            use Instruction::*;
            use Operand::*;
            use Ireg::*;
            use Freg::*;
            let mut asm = Assembler::new();
            $(asm.push_instruction($ins);)+
            asm.push_instruction(Instruction::Ret);
            let ops = asm.commit().unwrap();
            println!("{}", asm.buffer_fmt());
            let fun = callable!(ops, $jf);
            let ret = fun($($prms),+);
            assert_eq!(ret, $res);
        }
    }
}

// Tests
test!(add_reg_reg_i64;
    Mov(Ireg(RAX), Ireg(RCX)),
    Add(Ireg(RAX), Ireg(RDX));
    DyadI64;
    2, 3;
    5
);

test!(add_reg_imm_i64;
    Mov(Ireg(RAX), Iimm(9223372036854775801)),
    Add(Ireg(RAX), Ireg(RCX));
    MonadI64;
    6;
    9223372036854775807
);

test!(add_reg_reg_f64;
    Add(Freg(XMM0), Freg(XMM1));
    DyadF64;
    2.1, 3.2;
    5.300000000000001
);

test!(sub_reg_reg;
    Mov(Ireg(RAX), Ireg(RCX)),
    Sub(Ireg(RAX), Ireg(RDX));
    DyadI64;
    2, 3;
    -1
);

test!(mul_reg_reg;
    Mov(Ireg(RAX), Ireg(RCX)),
    Mul(Ireg(RAX), Ireg(RDX));
    DyadI64;
    2, 3;
    6
);

test!(mul_push_pop;
    Push(Ireg(RCX)),
    Push(Ireg(RDX)),
    Pop(Ireg(RAX)),
    Pop(Ireg(R9)),
    Mul(Ireg(RAX), Ireg(R9));
    DyadI64;
    2, 3;
    6
);
