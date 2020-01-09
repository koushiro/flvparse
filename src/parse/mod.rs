// Copyright 2019-2020 koushiro. Licensed under MIT.

// Parse the structure of the contents of FLV files.
// [The FLV File Format Spec](https://www.adobe.com/content/dam/acom/en/devnet/flv/video_file_format_spec_v10_1.pdf)

mod audio;
mod script;
mod video;

use nom::{be_u24, be_u32, be_u8, IResult};

pub use self::audio::*;
pub use self::script::*;
pub use self::video::*;
use crate::error::{Error, Result};

const FLV_HEADER_SIGNATURE: [u8; 3] = [0x46, 0x4c, 0x56];

///
pub fn parse(input: &[u8]) -> Result<FlvFile> {
    FlvFile::parse(input)
        .map_err(|_| Error::Parse)
        .map(|(_, flv)| flv)
}

/// The FLV file structure, including header and body.
#[derive(Debug, Clone, PartialEq)]
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
#[derive(Debug, Clone, Eq, PartialEq)]
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
#[derive(Debug, Clone, PartialEq)]
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
#[derive(Debug, Clone, PartialEq)]
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
            >> data: apply!(flv_tag_data, header.tag_type, header.data_size as usize)
            >> (FlvTag { header, data })
    )
}
//}

/// The tag header part of FLV tag.
#[derive(Debug, Clone, Eq, PartialEq)]
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
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
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
#[derive(Debug, Clone, PartialEq)]
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
        FlvTagType::Audio => map!(input, apply!(audio_tag, size), FlvTagData::Audio),
        FlvTagType::Video => map!(input, apply!(video_tag, size), FlvTagData::Video),
        FlvTagType::Script => map!(input, apply!(script_tag, size), FlvTagData::Script),
    }
}
//}

#[cfg(test)]
mod tests {
    use super::*;

    // Just use 3 tags of TEST_FLV_FILE:
    // 1. script tag
    // 2. video tag
    // 3. audio tag
    const TEST_FLV_FILE: &[u8] = include_bytes!("../assets/test.flv");
    const FLV_FILE_HEADER_LENGTH: usize = 9;
    const PREVIOUS_TAG_SIZE_LENGTH: usize = 4;
    const FLV_TAG_HEADER_LENGTH: usize = 11;
    const AUDIO_TAG_HEADER_LENGTH: usize = 1;
    const VIDEO_TAG_HEADER_LENGTH: usize = 1;

    #[test]
    fn test_parse_flv_file() {
        let flv_file = FlvFile::parse(&TEST_FLV_FILE[..]).unwrap().1;
        assert_eq!(
            flv_file.header,
            FlvFileHeader {
                signature: [0x46, 0x4c, 0x56],
                version: 1,
                flags: 0b0000_0101,
                has_audio: true,
                has_video: true,
                data_offset: 9,
            }
        );
        assert_eq!(flv_file.body.first_previous_tag_size, 0);
        assert_eq!(flv_file.body.tags[0].1, 11 + 1030);
        assert_eq!(flv_file.body.tags[1].1, 11 + 48);
        assert_eq!(flv_file.body.tags[2].1, 11 + 7);
    }

    #[test]
    fn test_flv_file_header() {
        let end = FLV_FILE_HEADER_LENGTH;
        println!(
            "flv file header = {:?}",
            FlvFileHeader::parse(&TEST_FLV_FILE[..end]).unwrap().1
        );
        assert_eq!(
            FlvFileHeader::parse(&TEST_FLV_FILE[..FLV_FILE_HEADER_LENGTH]),
            Ok((
                &b""[..],
                FlvFileHeader {
                    signature: [0x46, 0x4c, 0x56],
                    version: 1,
                    flags: 0b0000_0101,
                    has_audio: true,
                    has_video: true,
                    data_offset: 9,
                }
            ))
        );
    }

    #[test]
    fn test_flv_file_body() {
        let start = FLV_FILE_HEADER_LENGTH;
        let body = FlvFileBody::parse(&TEST_FLV_FILE[start..]).unwrap().1;
        assert_eq!(body.first_previous_tag_size, 0);
        assert_eq!(body.tags[0].1, 11 + 1030);
        assert_eq!(body.tags[1].1, 11 + 48);
        assert_eq!(body.tags[2].1, 11 + 7);
    }

    #[test]
    fn test_flv_tag() {
        // Just test audio tag (the third tag in TEST_FLV_FILE)
        let start = FLV_FILE_HEADER_LENGTH
            + PREVIOUS_TAG_SIZE_LENGTH
            + FLV_TAG_HEADER_LENGTH
            + 1030
            + PREVIOUS_TAG_SIZE_LENGTH
            + FLV_TAG_HEADER_LENGTH
            + 48
            + PREVIOUS_TAG_SIZE_LENGTH;
        let end: usize = start + FLV_TAG_HEADER_LENGTH + 7;
        println!(
            "flv tag = {:?}",
            FlvTag::parse(&TEST_FLV_FILE[start..end]).unwrap().1
        );
        assert_eq!(
            FlvTag::parse(&TEST_FLV_FILE[start..end]),
            Ok((
                &b""[..],
                FlvTag {
                    header: FlvTagHeader {
                        tag_type: FlvTagType::Audio, // 0x08
                        data_size: 7,                // 0x000007
                        timestamp: 0,                // 0x00000000
                        stream_id: 0,                // 0x000000
                    },
                    data: FlvTagData::Audio(AudioTag {
                        // 0xaf = 0b1010 1111, 1 byte
                        header: AudioTagHeader {
                            sound_format: SoundFormat::AAC, // 0b1010 = 10
                            sound_rate: SoundRate::_44KHZ,  // 0b11 = 3
                            sound_size: SoundSize::_16Bit,  // 0b01 = 1
                            sound_type: SoundType::Stereo,  // 0b01 = 1
                        },
                        // 0x0012 1056 e500, 6 bytes
                        body: AudioTagBody {
                            data: &b"\x00\x12\x10\x56\xe5\x00"[..],
                        },
                    })
                }
            ))
        );
    }

    #[test]
    fn test_flv_tag_header() {
        // The first tag starts at 9 bytes (flv file header size) + 4 bytes (previous tag size)
        // The size of tag header is 11 bytes.

        // script tag (the first tag in TEST_FLV_FILE)
        let mut start = FLV_FILE_HEADER_LENGTH + PREVIOUS_TAG_SIZE_LENGTH;
        let mut end = start + FLV_TAG_HEADER_LENGTH;
        println!(
            "flv tag header = {:?}",
            flv_tag_header(&TEST_FLV_FILE[start..end]).unwrap().1
        );
        assert_eq!(
            flv_tag_header(&TEST_FLV_FILE[start..end]),
            Ok((
                &b""[..],
                FlvTagHeader {
                    tag_type: FlvTagType::Script, // 0x12
                    data_size: 1030,              // 0x000406
                    timestamp: 0,                 // 0x00000000
                    stream_id: 0,                 // 0x000000
                }
            ))
        );

        // video tag (the second tag in TEST_FLV_FILE)
        start = end + 1030 + PREVIOUS_TAG_SIZE_LENGTH;
        end = start + FLV_TAG_HEADER_LENGTH;
        println!(
            "flv tag header = {:?}",
            flv_tag_header(&TEST_FLV_FILE[start..end]).unwrap().1
        );
        assert_eq!(
            flv_tag_header(&TEST_FLV_FILE[start..end]),
            Ok((
                &b""[..],
                FlvTagHeader {
                    tag_type: FlvTagType::Video, // 0x09
                    data_size: 48,               // 0x000030
                    timestamp: 0,                // 0x00000000
                    stream_id: 0,                // 0x000000
                }
            ))
        );

        // audio tag (the third tag in TEST_FLV_FILE)
        start = end + 48 + PREVIOUS_TAG_SIZE_LENGTH;
        end = start + FLV_TAG_HEADER_LENGTH;
        println!(
            "flv tag header = {:?}",
            flv_tag_header(&TEST_FLV_FILE[start..end]).unwrap().1
        );
        assert_eq!(
            flv_tag_header(&TEST_FLV_FILE[start..end]),
            Ok((
                &b""[..],
                FlvTagHeader {
                    tag_type: FlvTagType::Audio, // 0x08
                    data_size: 7,                // 0x000007
                    timestamp: 0,                // 0x00000000
                    stream_id: 0,                // 0x000000
                }
            ))
        );
    }

    #[test]
    fn test_flv_tag_data() {
        // Just test the audio tag (the third tag in TEST_FLV_FILE)
        let start = FLV_FILE_HEADER_LENGTH
            + PREVIOUS_TAG_SIZE_LENGTH
            + FLV_TAG_HEADER_LENGTH
            + 1030
            + PREVIOUS_TAG_SIZE_LENGTH
            + FLV_TAG_HEADER_LENGTH
            + 48
            + PREVIOUS_TAG_SIZE_LENGTH
            + FLV_TAG_HEADER_LENGTH;
        let end = start + 7;
        println!(
            "flv tag data = {:?}",
            FlvTagData::parse(&TEST_FLV_FILE[start..end], FlvTagType::Audio, 7)
                .unwrap()
                .1
        );
        assert_eq!(
            FlvTagData::parse(&TEST_FLV_FILE[start..end], FlvTagType::Audio, 7),
            Ok((
                &b""[..],
                FlvTagData::Audio(AudioTag {
                    // 0xaf = 0b1010 1111, 1 byte
                    header: AudioTagHeader {
                        sound_format: SoundFormat::AAC, // 0b1010 = 10
                        sound_rate: SoundRate::_44KHZ,  // 0b11 = 3
                        sound_size: SoundSize::_16Bit,  // 0b01 = 1
                        sound_type: SoundType::Stereo,  // 0b01 = 1
                    },
                    // 0x0012 1056 e500, 6 bytes
                    body: AudioTagBody {
                        data: &b"\x00\x12\x10\x56\xe5\x00"[..],
                    },
                })
            ))
        );
    }

    #[test]
    fn test_audio_tag() {
        // audio tag (the third tag in TEST_FLV_FILE)
        let start = FLV_FILE_HEADER_LENGTH
            + PREVIOUS_TAG_SIZE_LENGTH
            + FLV_TAG_HEADER_LENGTH
            + 1030
            + PREVIOUS_TAG_SIZE_LENGTH
            + FLV_TAG_HEADER_LENGTH
            + 48
            + PREVIOUS_TAG_SIZE_LENGTH
            + FLV_TAG_HEADER_LENGTH;
        let end = start + 7;
        println!(
            "audio tag = {:?}",
            audio_tag(&TEST_FLV_FILE[start..end], 7).unwrap().1
        );
        assert_eq!(
            audio_tag(&TEST_FLV_FILE[start..end], 7),
            Ok((
                &b""[..],
                AudioTag {
                    // 0xaf = 0b1010 1111, 1 byte
                    header: AudioTagHeader {
                        sound_format: SoundFormat::AAC, // 0b1010 = 10
                        sound_rate: SoundRate::_44KHZ,  // 0b11 = 3
                        sound_size: SoundSize::_16Bit,  // 0b01 = 1
                        sound_type: SoundType::Stereo,  // 0b01 = 1
                    },
                    // 0x0012 1056 e500, 6 bytes
                    body: AudioTagBody {
                        data: &b"\x00\x12\x10\x56\xe5\x00"[..],
                    },
                }
            ))
        );
    }

    #[test]
    fn test_audio_tag_header() {
        // audio tag (the third tag in TEST_FLV_FILE)
        let start = FLV_FILE_HEADER_LENGTH
            + PREVIOUS_TAG_SIZE_LENGTH
            + FLV_TAG_HEADER_LENGTH
            + 1030
            + PREVIOUS_TAG_SIZE_LENGTH
            + FLV_TAG_HEADER_LENGTH
            + 48
            + PREVIOUS_TAG_SIZE_LENGTH
            + FLV_TAG_HEADER_LENGTH;
        let end = start + AUDIO_TAG_HEADER_LENGTH;
        println!(
            "audio tag header = {:?}",
            audio_tag_header(&TEST_FLV_FILE[start..end], AUDIO_TAG_HEADER_LENGTH)
                .unwrap()
                .1
        );
        assert_eq!(
            audio_tag_header(&TEST_FLV_FILE[start..end], AUDIO_TAG_HEADER_LENGTH),
            Ok((
                &b""[..],
                // 0xaf = 0b1010 1111, 1 byte
                AudioTagHeader {
                    sound_format: SoundFormat::AAC, // 0b1010 = 10
                    sound_rate: SoundRate::_44KHZ,  // 0b11 = 3
                    sound_size: SoundSize::_16Bit,  // 0b01 = 1
                    sound_type: SoundType::Stereo,  // 0b01 = 1
                }
            ))
        );
    }

    #[test]
    fn test_audio_tag_body() {
        // audio tag (the third tag in TEST_FLV_FILE)
        let start = FLV_FILE_HEADER_LENGTH
            + PREVIOUS_TAG_SIZE_LENGTH
            + FLV_TAG_HEADER_LENGTH
            + 1030
            + PREVIOUS_TAG_SIZE_LENGTH
            + FLV_TAG_HEADER_LENGTH
            + 48
            + PREVIOUS_TAG_SIZE_LENGTH
            + FLV_TAG_HEADER_LENGTH
            + AUDIO_TAG_HEADER_LENGTH;
        let end = start + 7 - AUDIO_TAG_HEADER_LENGTH;
        println!(
            "audio tag body = {:?}",
            audio_tag_body(&TEST_FLV_FILE[start..end], 7 - AUDIO_TAG_HEADER_LENGTH)
                .unwrap()
                .1
        );
        assert_eq!(
            audio_tag_body(&TEST_FLV_FILE[start..end], 7 - AUDIO_TAG_HEADER_LENGTH),
            Ok((
                &b""[..],
                // 0x0012 1056 e500, 6 bytes
                AudioTagBody {
                    data: &b"\x00\x12\x10\x56\xe5\x00"[..],
                }
            ))
        );
    }

    #[test]
    fn test_video_tag() {
        // video tag header (the second tag in TEST_FLV_FILE)
        let start = FLV_FILE_HEADER_LENGTH
            + PREVIOUS_TAG_SIZE_LENGTH
            + FLV_TAG_HEADER_LENGTH
            + 1030
            + PREVIOUS_TAG_SIZE_LENGTH
            + FLV_TAG_HEADER_LENGTH;
        let end = start + 48;
        println!(
            "video tag = {:?}",
            video_tag(&TEST_FLV_FILE[start..end], 48).unwrap().1
        );
        assert_eq!(
            video_tag(&TEST_FLV_FILE[start..end], 48),
            Ok((
                &b""[..],
                VideoTag {
                    // 0x17 = 0b0001 0111, 1 byte
                    header: VideoTagHeader {
                        frame_type: FrameType::Key, // 0b0001 = 1
                        codec_id: CodecID::AVC,     // 0b0111 = 7
                    },
                    // 0x0000 0000 0164 0028 ffe1 001b 6764 0028 acd9 4078
                    //   0227 e5c0 4400 0003 0004 0000 0300 c03c 60c6 5801
                    //   0005 68eb ecf2 3c, 47 bytes
                    body: VideoTagBody {
                        data: &b"\x00\x00\x00\x00\x01\x64\x00\x28\xff\xe1\
                                 \x00\x1b\x67\x64\x00\x28\xac\xd9\x40\x78\
                                 \x02\x27\xe5\xc0\x44\x00\x00\x03\x00\x04\
                                 \x00\x00\x03\x00\xc0\x3c\x60\xc6\x58\x01\
                                 \x00\x05\x68\xeb\xec\xf2\x3c"[..],
                    },
                }
            ))
        );
    }

    #[test]
    fn test_video_tag_header() {
        // video tag header (the second tag in TEST_FLV_FILE)
        let start = FLV_FILE_HEADER_LENGTH
            + PREVIOUS_TAG_SIZE_LENGTH
            + FLV_TAG_HEADER_LENGTH
            + 1030
            + PREVIOUS_TAG_SIZE_LENGTH
            + FLV_TAG_HEADER_LENGTH;
        let end = start + VIDEO_TAG_HEADER_LENGTH;
        println!(
            "video tag header = {:?}",
            video_tag_header(&TEST_FLV_FILE[start..end], VIDEO_TAG_HEADER_LENGTH)
                .unwrap()
                .1
        );
        assert_eq!(
            video_tag_header(&TEST_FLV_FILE[start..end], VIDEO_TAG_HEADER_LENGTH),
            Ok((
                &b""[..],
                // 0x17 = 0b0001 0111, 1 byte
                VideoTagHeader {
                    frame_type: FrameType::Key, // 0b0001 = 1
                    codec_id: CodecID::AVC,     // 0b0111 = 7
                }
            ))
        );
    }

    #[test]
    fn test_video_tag_body() {
        // video tag (the second tag in TEST_FLV_FILE)
        let start = FLV_FILE_HEADER_LENGTH
            + PREVIOUS_TAG_SIZE_LENGTH
            + FLV_TAG_HEADER_LENGTH
            + 1030
            + PREVIOUS_TAG_SIZE_LENGTH
            + FLV_TAG_HEADER_LENGTH
            + VIDEO_TAG_HEADER_LENGTH;
        let end = start + 48 - VIDEO_TAG_HEADER_LENGTH;
        println!(
            "video tag body = {:?}",
            video_tag_body(&TEST_FLV_FILE[start..end], 48 - VIDEO_TAG_HEADER_LENGTH)
                .unwrap()
                .1
        );
        assert_eq!(
            video_tag_body(&TEST_FLV_FILE[start..end], 48 - VIDEO_TAG_HEADER_LENGTH),
            Ok((
                &b""[..],
                // 0x0000 0000 0164 0028 ffe1 001b 6764 0028 acd9 4078
                //   0227 e5c0 4400 0003 0004 0000 0300 c03c 60c6 5801
                //   0005 68eb ecf2 3c, 47 bytes
                VideoTagBody {
                    data: &b"\x00\x00\x00\x00\x01\x64\x00\x28\xff\xe1\
                             \x00\x1b\x67\x64\x00\x28\xac\xd9\x40\x78\
                             \x02\x27\xe5\xc0\x44\x00\x00\x03\x00\x04\
                             \x00\x00\x03\x00\xc0\x3c\x60\xc6\x58\x01\
                             \x00\x05\x68\xeb\xec\xf2\x3c"[..],
                }
            ))
        );
    }

    macro_rules! obj_prop {
        ($name:expr, $data:expr) => {
            ScriptDataObjectProperty {
                property_name: $name,
                property_data: $data,
            }
        };
    }

    #[test]
    fn test_script_tag() {
        // script tag (the first tag in TEST_FLV_FILE)
        let start = FLV_FILE_HEADER_LENGTH + PREVIOUS_TAG_SIZE_LENGTH + FLV_TAG_HEADER_LENGTH;
        let end = start + 1030;
        println!(
            "script tag = {:?}",
            script_tag(&TEST_FLV_FILE[start..end], 1030)
        );
        assert_eq!(
            script_tag(&TEST_FLV_FILE[start..end], 1030),
            Ok((
                &b""[..],
                ScriptTag {
                    name: "onMetaData",
                    value: ScriptDataValue::ECMAArray(vec![
                        obj_prop!(
                            "description",
                            ScriptDataValue::String(
                                "Codec by Bilibili XCode Worker v4.4.17(fixed_gap:False)"
                            )
                        ),
                        obj_prop!("metadatacreator", ScriptDataValue::String("Version 1.9")),
                        obj_prop!("hasKeyframes", ScriptDataValue::Boolean(true)),
                        obj_prop!("hasVideo", ScriptDataValue::Boolean(true)),
                        obj_prop!("hasAudio", ScriptDataValue::Boolean(true)),
                        obj_prop!("hasMetadata", ScriptDataValue::Boolean(true)),
                        obj_prop!("canSeekToEnd", ScriptDataValue::Boolean(true)),
                        obj_prop!("duration", ScriptDataValue::Number(194.517)),
                        obj_prop!("datasize", ScriptDataValue::Number(10168937.0)),
                        obj_prop!("videosize", ScriptDataValue::Number(2392510.0)),
                        obj_prop!("framerate", ScriptDataValue::Number(24.01543408360129)),
                        obj_prop!("videodatarate", ScriptDataValue::Number(94.09815112540193)),
                        obj_prop!("videocodecid", ScriptDataValue::Number(7.0)),
                        obj_prop!("width", ScriptDataValue::Number(1920.0)),
                        obj_prop!("height", ScriptDataValue::Number(1080.0)),
                        obj_prop!("audiosize", ScriptDataValue::Number(7724267.0)),
                        obj_prop!("audiodatarate", ScriptDataValue::Number(306.5355068580124)),
                        obj_prop!("audiocodecid", ScriptDataValue::Number(10.0)),
                        obj_prop!("audiosamplerate", ScriptDataValue::Number(3.0)),
                        obj_prop!("audiosamplesize", ScriptDataValue::Number(1.0)),
                        obj_prop!("stereo", ScriptDataValue::Boolean(true)),
                        obj_prop!("filesize", ScriptDataValue::Number(10169995.0)),
                        obj_prop!("lasttimestamp", ScriptDataValue::Number(194.375)),
                        obj_prop!("lastkeyframetimestamp", ScriptDataValue::Number(194.375)),
                        obj_prop!("lastkeyframelocation", ScriptDataValue::Number(10169975.0)),
                        obj_prop!(
                            "keyframes",
                            ScriptDataValue::Object(vec![
                                obj_prop!(
                                    "filepositions",
                                    ScriptDataValue::StrictArray(vec![
                                        ScriptDataValue::Number(1058.0),
                                        ScriptDataValue::Number(1143.0),
                                        ScriptDataValue::Number(371887.0),
                                        ScriptDataValue::Number(847626.0),
                                        ScriptDataValue::Number(1334735.0),
                                        ScriptDataValue::Number(1820692.0),
                                        ScriptDataValue::Number(2304839.0),
                                        ScriptDataValue::Number(2857985.0),
                                        ScriptDataValue::Number(3395640.0),
                                        ScriptDataValue::Number(3955507.0),
                                        ScriptDataValue::Number(4448601.0),
                                        ScriptDataValue::Number(4917339.0),
                                        ScriptDataValue::Number(5380323.0),
                                        ScriptDataValue::Number(5862615.0),
                                        ScriptDataValue::Number(6364648.0),
                                        ScriptDataValue::Number(6867232.0),
                                        ScriptDataValue::Number(7414669.0),
                                        ScriptDataValue::Number(7950581.0),
                                        ScriptDataValue::Number(8594010.0),
                                        ScriptDataValue::Number(9433239.0),
                                        ScriptDataValue::Number(10088872.0),
                                        ScriptDataValue::Number(10169975.0),
                                    ])
                                ),
                                obj_prop!(
                                    "times",
                                    ScriptDataValue::StrictArray(vec![
                                        ScriptDataValue::Number(0.0),
                                        ScriptDataValue::Number(0.0),
                                        ScriptDataValue::Number(10.0),
                                        ScriptDataValue::Number(20.0),
                                        ScriptDataValue::Number(30.0),
                                        ScriptDataValue::Number(40.0),
                                        ScriptDataValue::Number(50.0),
                                        ScriptDataValue::Number(60.0),
                                        ScriptDataValue::Number(70.0),
                                        ScriptDataValue::Number(80.0),
                                        ScriptDataValue::Number(90.0),
                                        ScriptDataValue::Number(100.0),
                                        ScriptDataValue::Number(110.0),
                                        ScriptDataValue::Number(120.0),
                                        ScriptDataValue::Number(130.0),
                                        ScriptDataValue::Number(140.0),
                                        ScriptDataValue::Number(150.0),
                                        ScriptDataValue::Number(160.0),
                                        ScriptDataValue::Number(170.0),
                                        ScriptDataValue::Number(180.0),
                                        ScriptDataValue::Number(190.0),
                                        ScriptDataValue::Number(194.375),
                                    ])
                                ),
                            ])
                        ),
                    ]),
                }
            ))
        );
    }

    #[test]
    fn test_script_data_date() {
        let input = &b"\x00\x00\x00\x00\x00\x00\x00\x00\
                             \x00\x08Remain"[..];
        println!(
            "script data date = {:?}",
            script_data_date(input).unwrap().1
        );
        assert_eq!(
            script_data_date(input),
            Ok((
                &b"Remain"[..],
                ScriptDataDate {
                    date_time: 0.0,
                    local_date_time_offset: 8,
                }
            ))
        );
    }

    #[test]
    fn test_script_data_long_string() {
        let input = &b"\x00\x00\x00\x0b\
                             Long StringRemain"[..];
        println!(
            "script data long string = {:?}",
            script_data_long_string(input).unwrap().1
        );
        assert_eq!(
            script_data_long_string(input),
            Ok((&b"Remain"[..], "Long String"))
        );
    }
}
