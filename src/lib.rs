// Copyright 2019-2021 koushiro. Licensed under MIT.

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

pub use self::parse::*;

pub use nom::{
    error::{Error, ErrorKind},
    Err as NomErr, IResult, Needed,
};
