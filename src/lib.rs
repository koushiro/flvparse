// Copyright 2019-2020 koushiro. Licensed under MIT.

//! # flvparser
//!
//! A FLV file parser written in Rust with nom.

#![deny(missing_docs)]

#[macro_use]
extern crate nom;

mod error;
mod parse;

pub use self::error::{Error, Result};
pub use self::parse::*;

///
pub fn parse(input: &[u8]) -> Result<FlvFile> {
    match FlvFile::parse(input) {
        Ok((_output, flv)) => Ok(flv),
        Err(_) => Err(Error::Parse)
    }
}
