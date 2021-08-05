#[macro_use]
extern crate pest_derive;

pub mod gen;
pub mod parser;

pub use gen::Codegen;
pub use parser::{parse, Command};
