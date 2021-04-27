#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate lazy_static;

pub mod api;
pub mod challenge;
pub mod error;
pub mod utils;

pub static MAGIC: &'static str = "This program cannot be run in DOS mode\0";
