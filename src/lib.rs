// Copyright 2019-2020 koushiro. Licensed under MIT.

//! # flvparser
//!
//! A FLV file parser written in Rust with nom.
//!
//! ## Example
//!
//! ```rust
//! use flvparser::*;
//!
//! let flv_file = parse(include_bytes!("../assets/test.flv")).unwrap();
//! assert_eq!(
//!     flv_file.header,
//!     FlvFileHeader {
//!         signature: [0x46, 0x4c, 0x56],
//!         version: 1,
//!         flags: 0b0000_0101,
//!         has_audio: true,
//!         has_video: true,
//!         data_offset: 9,
//!     }
//! );
//! assert_eq!(flv_file.body.first_previous_tag_size, 0);
//! assert_eq!(flv_file.body.tags[0].1, 11 + 1030);
//! assert_eq!(flv_file.body.tags[1].1, 11 + 48);
//! assert_eq!(flv_file.body.tags[2].1, 11 + 7);
//! ```

#![deny(missing_docs)]

#[macro_use]
extern crate nom;

mod error;
mod parse;

pub use self::error::{Error, Result};
pub use self::parse::*;
