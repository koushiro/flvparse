// Copyright 2019-2020 koushiro. Licensed under MIT.

#![allow(clippy::unreadable_literal)]

use flvparse::*;

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
        FlvTagHeader::parse(&TEST_FLV_FILE[start..end]).unwrap().1
    );
    assert_eq!(
        FlvTagHeader::parse(&TEST_FLV_FILE[start..end]),
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
        FlvTagHeader::parse(&TEST_FLV_FILE[start..end]).unwrap().1
    );
    assert_eq!(
        FlvTagHeader::parse(&TEST_FLV_FILE[start..end]),
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
        FlvTagHeader::parse(&TEST_FLV_FILE[start..end]).unwrap().1
    );
    assert_eq!(
        FlvTagHeader::parse(&TEST_FLV_FILE[start..end]),
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
        AudioTag::parse(&TEST_FLV_FILE[start..end], 7).unwrap().1
    );
    assert_eq!(
        AudioTag::parse(&TEST_FLV_FILE[start..end], 7),
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
        AudioTagHeader::parse(&TEST_FLV_FILE[start..end], AUDIO_TAG_HEADER_LENGTH)
            .unwrap()
            .1
    );
    assert_eq!(
        AudioTagHeader::parse(&TEST_FLV_FILE[start..end], AUDIO_TAG_HEADER_LENGTH),
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
        AudioTagBody::parse(&TEST_FLV_FILE[start..end], 7 - AUDIO_TAG_HEADER_LENGTH)
            .unwrap()
            .1
    );
    assert_eq!(
        AudioTagBody::parse(&TEST_FLV_FILE[start..end], 7 - AUDIO_TAG_HEADER_LENGTH),
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
        VideoTag::parse(&TEST_FLV_FILE[start..end], 48).unwrap().1
    );
    assert_eq!(
        VideoTag::parse(&TEST_FLV_FILE[start..end], 48),
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
        VideoTagHeader::parse(&TEST_FLV_FILE[start..end], VIDEO_TAG_HEADER_LENGTH)
            .unwrap()
            .1
    );
    assert_eq!(
        VideoTagHeader::parse(&TEST_FLV_FILE[start..end], VIDEO_TAG_HEADER_LENGTH),
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
        VideoTagBody::parse(&TEST_FLV_FILE[start..end], 48 - VIDEO_TAG_HEADER_LENGTH)
            .unwrap()
            .1
    );
    assert_eq!(
        VideoTagBody::parse(&TEST_FLV_FILE[start..end], 48 - VIDEO_TAG_HEADER_LENGTH),
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
    ($name:expr, $value:expr) => {
        ScriptDataObjectProperty {
            name: $name,
            value: $value,
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
        ScriptTag::parse(&TEST_FLV_FILE[start..end], 1030)
    );
    assert_eq!(
        ScriptTag::parse(&TEST_FLV_FILE[start..end], 1030),
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
    let input = &b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x08Remain"[..];
    println!(
        "script data date = {:?}",
        ScriptDataValue::parse_date(input).unwrap().1
    );
    assert_eq!(
        ScriptDataValue::parse_date(input),
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
    let input = &b"\x00\x00\x00\x0bLong StringRemain"[..];
    println!(
        "script data long string = {:?}",
        ScriptDataValue::parse_long_string(input).unwrap().1
    );
    assert_eq!(
        ScriptDataValue::parse_long_string(input),
        Ok((&b"Remain"[..], "Long String"))
    );
}
