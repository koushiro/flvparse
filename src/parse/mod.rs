// Parse the structure of the contents of FLV files.
// [The FLV File Format Spec](https://www.adobe.com/content/dam/acom/en/devnet/flv/video_file_format_spec_v10_1.pdf)

mod audio;
mod script;
mod video;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use nom::{
    Err as NomErr, IResult, Parser,
    bytes::streaming::{tag, take},
    combinator::complete,
    error::{Error, ErrorKind},
    multi::many0,
    number::streaming::{be_u8, be_u32},
};

pub use self::{audio::*, script::*, video::*};

const FLV_HEADER_SIGNATURE: [u8; 3] = [0x46, 0x4c, 0x56];

fn be_u24(input: &[u8]) -> IResult<&[u8], u32> {
    let (input, bytes) = take(3usize)(input)?;
    let value = (u32::from(bytes[0]) << 16) | (u32::from(bytes[1]) << 8) | u32::from(bytes[2]);
    Ok((input, value))
}

/// The FLV file structure, including header and body.
#[derive(Clone, Debug, PartialEq)]
pub struct FlvFile<'a> {
    /// The header of FLV file.
    pub header: FlvFileHeader,
    /// The body of FLV file.
    pub body: FlvFileBody<'a>,
}

impl<'a> FlvFile<'a> {
    /// Parse FLV file.
    pub fn parse(input: &'a [u8]) -> IResult<&'a [u8], FlvFile<'a>> {
        let (input, header) = FlvFileHeader::parse(input)?;
        let (input, body) = FlvFileBody::parse(input)?;
        Ok((input, FlvFile { header, body }))
    }
}

/// The header part of FLV file.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct FlvFileHeader {
    /// Signature bytes are always "FLV" (0x46, 0x4c, 0x56).
    pub signature: [u8; 3],
    /// File version (0x01 for FLV version 1).
    pub version: u8,
    /// TypeFlagsReserved   5 bits  Shall be 0.
    /// TypeFlagsAudio      1 bit   1 = Audio tags are present.
    /// TypeFlagsReserved   1 bit   Shall be 0.
    /// TypeFlagsVideo      1 bit   1 = Video tags are present.
    pub flags: u8,
    /// The flag that represents whether the audio exists in FLV file.
    pub has_audio: bool,
    /// The flag that represents whether the video exists in FLV file.
    pub has_video: bool,
    /// The length of this header in bytes, usually has a value of 9 for FLV version 1.
    pub data_offset: u32,
}

impl FlvFileHeader {
    /// Parse FLV file header.
    pub fn parse(input: &[u8]) -> IResult<&[u8], FlvFileHeader> {
        let (input, _) = tag(FLV_HEADER_SIGNATURE.as_slice())(input)?;
        let (input, version) = be_u8(input)?;
        let (input, flags) = be_u8(input)?;
        let (input, data_offset) = be_u32(input)?;

        Ok((
            input,
            FlvFileHeader {
                signature: FLV_HEADER_SIGNATURE,
                version,
                flags,
                has_audio: flags & 4 == 4,
                has_video: flags & 1 == 1,
                data_offset,
            },
        ))
    }
}

/// The body part of FLV file.
#[derive(Clone, Debug, PartialEq)]
pub struct FlvFileBody<'a> {
    /// The size of the first previous tag is always 0.
    pub first_previous_tag_size: u32,
    /// FLV Tag and the size of the tag.
    pub tags: Vec<(FlvTag<'a>, u32)>,
}

impl<'a> FlvFileBody<'a> {
    // https://github.com/Geal/nom/issues/790 - many0 returns Incomplete in weird cases.
    /// Parse FLV file body.
    pub fn parse(input: &'a [u8]) -> IResult<&'a [u8], FlvFileBody<'a>> {
        let (input, first_previous_tag_size) = be_u32(input)?;
        let (input, tags) = many0(complete((|i| FlvTag::parse(i), be_u32))).parse(input)?;

        Ok((input, FlvFileBody { first_previous_tag_size, tags }))
    }
}

/// The FLV tag has three types: `script tag`, `audio tag` and `video tag`.
/// Each tag contains tag header and tag data.
/// The structure of each type of tag header is the same.
#[derive(Clone, Debug, PartialEq)]
pub struct FlvTag<'a> {
    /// The header part of FLV tag.
    pub header: FlvTagHeader,
    /// Data specific for each media type:
    /// * 8 = audio data.
    /// * 9 = video data.
    /// * 18 = script data.
    pub data: FlvTagData<'a>,
}

impl<'a> FlvTag<'a> {
    /// Parse FLV tag.
    pub fn parse(input: &'a [u8]) -> IResult<&'a [u8], FlvTag<'a>> {
        let (input, header) = FlvTagHeader::parse(input)?;
        let (input, data) = FlvTagData::parse(input, header.tag_type, header.data_size as usize)?;
        Ok((input, FlvTag { header, data }))
    }
}

/// The tag header part of FLV tag.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct FlvTagHeader {
    /// Reserved    2 bits  Reserved for FMS, should be 0.
    /// Filter      1 bit   Indicates if packets are filtered.
    ///                     0 = No pre-processing required
    ///                     1 = Pre-processing (Such as decryption) of the packet
    ///                         is required before it can be rendered.
    /// TagType     5 bits  The type of contents in this tag,
    ///                     8 = audio, 9 = video, 18 = script.
    pub tag_type: FlvTagType,
    /// The size of the tag's data part, 3 bytes.
    pub data_size: u32,
    /// The timestamp (in milliseconds) of the tag,
    /// Timestamp (3 bytes) + TimestampExtended (1byte).
    pub timestamp: u32,
    /// The id of stream is always 0, 3 bytes.
    pub stream_id: u32,
}

/// The type of FLV tag.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum FlvTagType {
    /// Audio tag type.
    Audio = 0x08,
    /// Video tag type.
    Video = 0x09,
    /// Script tag type.
    Script = 0x18,
}

impl FlvTagHeader {
    /// Parse FLV tag header.
    pub fn parse(input: &[u8]) -> IResult<&[u8], FlvTagHeader> {
        let (input, tag_type_byte) = be_u8(input)?;
        let tag_type = match tag_type_byte {
            8 => FlvTagType::Audio,
            9 => FlvTagType::Video,
            18 => FlvTagType::Script,
            _ => return Err(NomErr::Error(Error::new(input, ErrorKind::Switch))),
        };
        let (input, data_size) = be_u24(input)?;
        let (input, timestamp) = be_u24(input)?;
        let (input, timestamp_extended) = be_u8(input)?;
        let (input, stream_id) = be_u24(input)?;

        Ok((
            input,
            FlvTagHeader {
                tag_type,
                data_size,
                timestamp: (u32::from(timestamp_extended) << 24) + timestamp,
                stream_id,
            },
        ))
    }
}

/// The tag data part of FLV tag.
#[derive(Clone, Debug, PartialEq)]
pub enum FlvTagData<'a> {
    /// Audio tag data.
    Audio(AudioTag<'a>),
    /// Video tag data.
    Video(VideoTag<'a>),
    /// Script tag data.
    Script(ScriptTag<'a>),
}

impl<'a> FlvTagData<'a> {
    /// Parse FLV tag data.
    pub fn parse(
        input: &'a [u8],
        tag_type: FlvTagType,
        size: usize,
    ) -> IResult<&'a [u8], FlvTagData<'a>> {
        match tag_type {
            FlvTagType::Audio => {
                let (input, tag) = AudioTag::parse(input, size)?;
                Ok((input, FlvTagData::Audio(tag)))
            },
            FlvTagType::Video => {
                let (input, tag) = VideoTag::parse(input, size)?;
                Ok((input, FlvTagData::Video(tag)))
            },
            FlvTagType::Script => {
                let (input, tag) = ScriptTag::parse(input, size)?;
                Ok((input, FlvTagData::Script(tag)))
            },
        }
    }
}
