/// # flvparser
///
/// A FLV file parser written in Rust with nom.
///
/// ## Example
///
/// ```rust
/// extern crate flvparser;
/// use flvparser::*;
///
/// fn main() {
///     let flv_file = FLVParser::parse(include_bytes!("../assets/test.flv")).unwrap();
///     assert_eq!(
///         flv_file.header,
///         FLVFileHeader {
///             signature: [0x46, 0x4c, 0x56],
///             version: 1,
///             flags: 0b0000_0101,
///             has_audio: true,
///             has_video: true,
///             data_offset: 9,
///         }
///     );
///     assert_eq!(flv_file.body.first_previous_tag_size, 0);
///     assert_eq!(flv_file.body.tags[0].1, 11 + 1030);
///     assert_eq!(flv_file.body.tags[1].1, 11 + 48);
///     assert_eq!(flv_file.body.tags[2].1, 11 + 7);
/// }
/// ```

#[macro_use]
extern crate nom;

pub mod flv_parser;
pub use flv_parser::*;
