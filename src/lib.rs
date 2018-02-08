#![feature(unique)]

extern crate core;
extern crate memmap;
extern crate fnv;

#[derive(Debug, Clone, Copy)]
pub enum Error {
    MmapCreate,
    MmapSetMode,
    EmptyBuffer,
    InvalidOperation,
    Serialize,
    UnknownLabel,
    Nyi
}

#[macro_use]
pub mod ops;
#[macro_use]
pub mod x64;


