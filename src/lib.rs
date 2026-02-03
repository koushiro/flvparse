//! # flvparse
//!
//! A FLV format parsing library written in Rust with nom.

#![deny(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

mod parse;

pub use nom::{
    Err as NomErr, IResult, Needed,
    error::{Error, ErrorKind},
};

pub use self::parse::*;
