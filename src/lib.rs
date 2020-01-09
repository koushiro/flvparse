// Copyright 2019-2020 koushiro. Licensed under MIT.

//! # flvparser
//!
//! A FLV file parser written in Rust with nom.

#![deny(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

#[macro_use]
extern crate nom;

mod error;
mod parse;

pub use self::error::{Error, Result};
pub use self::parse::*;

/// A helper function for parsing FLV format.
pub fn parse(input: &[u8]) -> Result<FlvFile> {
    match FlvFile::parse(input) {
        Ok((_output, flv)) => Ok(flv),
        Err(_) => Err(Error::Parse),
    }
}
