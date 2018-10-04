#[macro_use]
extern crate nom;

pub mod flv_parser;

pub use flv_parser::{FLVFile, parse_flv_file, FLVTagType};
