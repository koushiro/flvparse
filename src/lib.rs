//! # flvparse
//!
//! A FLV format parsing library written in Rust with nom.

#![deny(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

#[macro_use]
extern crate nom;

mod parse;

pub use nom::{
    Err as NomErr, IResult, Needed,
    error::{Error, ErrorKind},
};

pub use self::parse::*;
