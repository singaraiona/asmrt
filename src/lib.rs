#![feature(unique)]

extern crate core;
extern crate memmap;

#[derive(Debug, Clone, Copy)]
pub enum Error {
    MmapCreate,
    MmapSetMode,
    EmptyBuffer,
    InvalidOperation,
    Nyi
}

#[macro_use]
pub mod defs;
#[macro_use]
pub mod x64;


