// Copyright 2019-2020 koushiro. Licensed under MIT.

// Parse the structure of the contents of FLV files.
// [The FLV File Format Spec](https://www.adobe.com/content/dam/acom/en/devnet/flv/video_file_format_spec_v10_1.pdf)

mod audio;
mod script;
mod video;

use nom::{
    number::streaming::{be_u24, be_u32, be_u8},
    IResult,
};

pub use self::audio::*;
pub use self::script::*;
pub use self::video::*;

const FLV_HEADER_SIGNATURE: [u8; 3] = [0x46, 0x4c, 0x56];

/// The FLV file structure, including header and body.
#[derive(Clone, Debug, PartialEq)]
pub struct FlvFile<'a> {
    /// The header of FLV file.
    pub header: FlvFileHeader,
    /// The body of FLV file.
    pub body: FlvFileBody<'a>,
}

impl<'a> FlvFile<'a> {
    ///
    pub fn parse(input: &'a [u8]) -> IResult<&'a [u8], FlvFile<'a>> {
        do_parse!(
            input,
            header: flv_file_header >> body: flv_file_body >> (FlvFile { header, body })
        )
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

//impl FlvFileHeader {
///
pub fn flv_file_header(input: &[u8]) -> IResult<&[u8], FlvFileHeader> {
    do_parse!(
        input,
        // FLV Signature
        tag!(FLV_HEADER_SIGNATURE)  >>
            // FLV File Version
            version:     be_u8          >>
            // Flags
            flags:       be_u8          >>
            // The length of this header in bytes
            data_offset: be_u32         >>

            (FlvFileHeader {
                signature: FLV_HEADER_SIGNATURE,
                version,
                flags,
                has_audio: flags & 4 == 4,
                has_video: flags & 1 == 1,
                data_offset,
            })
    )
}
//}

/// The body part of FLV file.
#[derive(Clone, Debug, PartialEq)]
pub struct FlvFileBody<'a> {
    /// The size of the first previous tag is always 0.
    pub first_previous_tag_size: u32,
    /// FLV Tag and the size of the tag.
    pub tags: Vec<(FlvTag<'a>, u32)>,
}

//impl<'a> FlvFileBody<'a> {
// https://github.com/Geal/nom/issues/790 - many0 returns Incomplete in weird cases.
///
pub fn flv_file_body(input: &[u8]) -> IResult<&[u8], FlvFileBody> {
    do_parse!(
        input,
        // The first previous tag size.
        first_previous_tag_size: be_u32                    >>
            // FLV Tag and the size of the tag.
            tags: many0!(complete!(tuple!(flv_tag, be_u32)))   >>

            (FlvFileBody { first_previous_tag_size, tags })
    )
}
//}

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

//impl<'a> FlvTag<'a> {
///
pub fn flv_tag(input: &[u8]) -> IResult<&[u8], FlvTag> {
    do_parse!(
        input,
        header: flv_tag_header
            >> data: call!(flv_tag_data, header.tag_type, header.data_size as usize)
            >> (FlvTag { header, data })
    )
}
//}

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
    /// The timestamp (in milliseconds) of the tag, Timestamp (3 bytes) + TimestampExtended (1 byte).
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

//impl FlvTagHeader {
///
pub fn flv_tag_header(input: &[u8]) -> IResult<&[u8], FlvTagHeader> {
    do_parse!(
        input,
        // Tag Type
        tag_type: switch!(be_u8,
                8  => value!(FlvTagType::Audio) |
                9  => value!(FlvTagType::Video) |
                18 => value!(FlvTagType::Script)
            )                           >>
            // The size of the tag's data part
            data_size:          be_u24  >>
            // The timestamp (in milliseconds) of the tag
            timestamp:          be_u24  >>
            // Extension of the timestamp field to form a SI32 value
            timestamp_extended: be_u8   >>
            // The id of stream
            stream_id:          be_u24  >>
            (FlvTagHeader {
                tag_type,
                data_size,
                timestamp: (u32::from(timestamp_extended) << 24) + timestamp,
                stream_id,
            })
    )
}
//}

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

//impl<'a> FlvTagData<'a> {
///
pub fn flv_tag_data(input: &[u8], tag_type: FlvTagType, size: usize) -> IResult<&[u8], FlvTagData> {
    match tag_type {
        FlvTagType::Audio => map!(input, call!(audio_tag, size), FlvTagData::Audio),
        FlvTagType::Video => map!(input, call!(video_tag, size), FlvTagData::Video),
        FlvTagType::Script => map!(input, call!(script_tag, size), FlvTagData::Script),
    }
}
//}
