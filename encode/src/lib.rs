#![feature(plugin_registrar, rustc_private)]

extern crate syntax;
extern crate rustc;
extern crate rustc_plugin;

use rustc_plugin::registry::Registry;
use syntax::ext::base::{SyntaxExtension, ExtCtxt, MacResult, DummyResult};
use syntax::ext::build::AstBuilder;
use syntax::codemap::{Span, Spanned};
use syntax::ast;
use syntax::util::small_vector::SmallVector;
use syntax::parse::parser::Parser;
use syntax::parse::PResult;
use syntax::symbol::Symbol;
use syntax::parse::token;
use syntax::tokenstream::TokenTree;
use syntax::ptr::P;

fn encode(cx: &mut ExtCtxt, sp: Span, args: &[TokenTree]) -> Box<MacResult + 'static> {
    let mut parser = cx.new_parser_from_tts(args);
    //for a in args {
        //println!("A: {:?}", a);
    //}
    //compile(cx, parser)
    return DummyResult::any(sp);
}

fn compile<'a>(cx: &mut ExtCtxt, parser: &mut Parser<'a>) -> PResult<'a, Vec<u8>> {
   Ok(vec![1,2,3,4,5,6])
}

#[plugin_registrar]
pub fn registrar(reg: &mut Registry) {
    reg.register_macro("encode", encode);
}
