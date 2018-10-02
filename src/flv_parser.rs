// https://www.adobe.com/content/dam/acom/en/devnet/flv/video_file_format_spec_v10_1.pdf  -- Annex E. The FLV File Format

use std::str;

use nom::{
    be_u8, be_u16, be_u24, be_u32, be_i16, be_i24, be_f64,
    IResult, Err as NomErr, Needed
};

#[derive(Debug, Clone, PartialEq)]
pub struct FLVFile<'a> {
    pub header: FLVFileHeader,
    pub body: FLVFileBody<'a>,
}

pub fn parse_flv_file(input: &[u8]) -> IResult<&[u8], FLVFile> {
    do_parse!(
        input,
        // FLV file header.
        header: flv_file_header >>
        // FLV file body.
        body:   flv_file_body   >>
        (FLVFile {
            header,
            body,
        })
    )
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FLVFileHeader {
    /// Signature bytes are always "FLV" (0x46, 0x4c, 0x56).
    pub signature: [u8; 3],
    /// File version (0x01 for FLV version 1).
    pub version: u8,
    /// TypeFlagsReserved   5 bits  Shall be 0.
    /// TypeFlagsAudio      1 bit   1 = Audio tags are present.
    /// TypeFlagsReserved   1 bit   Shall be 0.
    /// TypeFlagsVideo      1 bit   1 = Video tags are present.
    pub flags: u8,
    pub has_audio: bool,
    pub has_video: bool,
    /// The length of this header in bytes, usually has a value of 9 for FLV version 1.
    pub data_offset: u32,
}

static FLV_HEADER_SIGNATURE: &'static [u8] = &[0x46, 0x4c, 0x56];
pub fn flv_file_header(input: &[u8]) -> IResult<&[u8], FLVFileHeader> {
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
        (FLVFileHeader {
            signature: [0x46, 0x4c, 0x56],
            version,
            flags,
            has_audio: flags & 4 == 4,
            has_video: flags & 1 == 1,
            data_offset,
        })
    )
}

#[derive(Debug, Clone, PartialEq)]
pub struct FLVFileBody<'a> {
    /// The size of the first previous tag is always 0.
    pub first_previous_tag_size: u32,
    /// FLV Tag and the size of the tag.
    pub tags: Vec<(FLVTag<'a>, u32)>,
}

// https://github.com/Geal/nom/issues/790 - many0 returns Incomplete in weird cases.
pub fn flv_file_body(input: &[u8]) -> IResult<&[u8], FLVFileBody> {
    do_parse!(
        input,
        // The first previous tag size.
        first_previous_tag_size: be_u32                    >>
        // FLV Tag and the size of the tag.
        tags: many0!(complete!(tuple!(flv_tag, be_u32)))   >>
        (FLVFileBody {
            first_previous_tag_size,
            tags,
        })
    )
}

#[derive(Debug, Clone, PartialEq)]
pub struct FLVTag<'a> {
    pub header: FLVTagHeader,
    /// Data specific for each media type.
    /// 8 = audio data.
    /// 9 = video data.
    /// 18 = script data.
    pub data: FLVTagData<'a>,
}

pub fn flv_tag(input: &[u8]) -> IResult<&[u8], FLVTag> {
    do_parse!(
        input,
        // FLVTagHeader
        header: flv_tag_header                                  >>
        // FLVTagData
        data:   apply!(flv_tag_data,
                    header.tag_type, header.data_size as usize) >>
        (FLVTag {
            header,
            data,
        })
    )
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FLVTagHeader {
    /// Reserved    2 bits  Reserved for FMS, should be 0.
    /// Filter      1 bit   Indicates if packets are filtered.
    ///                     0 = No pre-processing required
    ///                     1 = Pre-processing (Such as decryption) of the packet
    ///                         is required before it can be rendered.
    /// TagType     5 bits  The type of contents in this tag,
    ///                     8 = audio, 9 = video, 18 = script.
    pub tag_type: FLVTagType,
    /// The size of the tag's data part, 3 bytes.
    pub data_size: u32,
    /// The timestamp (in milliseconds) of the tag,
    /// Timestamp (3 bytes) + TimestampExtended (1 byte).
    pub timestamp: u32,
    /// The id of stream is always 0, 3 bytes.
    pub stream_id: u32,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum FLVTagType {
    Audio,  // 0x08
    Video,  // 0x09
    Script, // 0x18
}

pub fn flv_tag_header(input: &[u8]) -> IResult<&[u8], FLVTagHeader> {
    do_parse!(
        input,
        // Tag Type
        tag_type: switch!(be_u8,
            8  => value!(FLVTagType::Audio) |
            9  => value!(FLVTagType::Video) |
            18 => value!(FLVTagType::Script)
        )                           >>
        // The size of the tag's data part
        data_size:          be_u24  >>
        // The timestamp (in milliseconds) of the tag
        timestamp:          be_u24  >>
        // Extension of the timestamp field to form a SI32 value
        timestamp_extended: be_u8   >>
        // The id of stream
        stream_id:          be_u24  >>
        (FLVTagHeader {
            tag_type,
            data_size,
            timestamp: (u32::from(timestamp_extended) << 24) + timestamp,
            stream_id,
        })
    )
}

#[derive(Debug, Clone, PartialEq)]
pub enum FLVTagData<'a> {
    Audio(AudioTag<'a>),
    Video(VideoTag<'a>),
    Script(ScriptTag<'a>),
}

pub fn flv_tag_data(input: &[u8], tag_type: FLVTagType, size: usize) -> IResult<&[u8], FLVTagData> {
    match tag_type {
        FLVTagType::Audio => map!(input, apply!(audio_tag, size), FLVTagData::Audio),
        FLVTagType::Video => map!(input, apply!(video_tag, size), FLVTagData::Video),
        FLVTagType::Script => map!(input, apply!(script_tag, size), FLVTagData::Script),
    }
}

// ----------------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq)]
pub struct AudioTag<'a> {
    pub header: AudioTagHeader, // 8 bits.
    pub body: AudioTagBody<'a>,
}

pub fn audio_tag(input: &[u8], size: usize) -> IResult<&[u8], AudioTag> {
    do_parse!(
        input,
        // AudioTagHeader
        header: apply!(audio_tag_header, size)      >>
        // AudioTagBody
        body:   apply!(audio_tag_body, size - 1)    >>
        (AudioTag {
            header,
            body,
        })
    )
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct AudioTagHeader {
    pub sound_format: SoundFormat, // 4 bits.
    pub sound_rate: SoundRate,     // 2 bits.
    pub sound_size: SoundSize,     // 1 bit.
    pub sound_type: SoundType,     // 1 bit.
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SoundFormat {
    PcmPlatformEndian,   // 0
    ADPCM,               // 1
    MP3,                 // 2
    PcmLittleEndian,     // 3
    Nellymoser16kHzMono, // 4
    Nellymoser8kHzMono,  // 5
    Nellymoser,          // 6
    PcmALaw,             // 7
    PcmMuLaw,            // 8
    Reserved,            // 9
    AAC,                 // 10, MPEG-4 Part3 AAC
    Speex,               // 11
    MP3_8kHz,            // 14
    DeviceSpecific,      // 15
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SoundRate {
    _5_5KHZ, // 0
    _11KHZ,  // 1
    _22KHZ,  // 2
    _44KHZ,  // 3
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SoundSize {
    _8Bit,  // 0
    _16Bit, // 1
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SoundType {
    Mono,   // 0
    Stereo, // 1
}

pub fn audio_tag_header(input: &[u8], size: usize) -> IResult<&[u8], AudioTagHeader> {
    if size < 1 {
        return Err(NomErr::Incomplete(Needed::Size(1)));
    }

    let (remain, (sound_format, sound_rate, sound_size, sound_type)) = try_parse!(
        input,
        bits!(tuple!(
            switch!(take_bits!(u8, 4),
                    0  => value!(SoundFormat::PcmPlatformEndian)    |
                    1  => value!(SoundFormat::ADPCM)                |
                    2  => value!(SoundFormat::MP3)                  |
                    3  => value!(SoundFormat::PcmLittleEndian)      |
                    4  => value!(SoundFormat::Nellymoser16kHzMono)  |
                    5  => value!(SoundFormat::Nellymoser8kHzMono)   |
                    6  => value!(SoundFormat::Nellymoser)           |
                    7  => value!(SoundFormat::PcmALaw)              |
                    8  => value!(SoundFormat::PcmMuLaw)             |
                    9  => value!(SoundFormat::Reserved)             |
                    10 => value!(SoundFormat::AAC)                  |
                    11 => value!(SoundFormat::Speex)                |
                    14 => value!(SoundFormat::MP3_8kHz)             |
                    15 => value!(SoundFormat::DeviceSpecific)
            ),
            switch!(take_bits!(u8, 2),
                    0 => value!(SoundRate::_5_5KHZ) |
                    1 => value!(SoundRate::_11KHZ)  |
                    2 => value!(SoundRate::_22KHZ)  |
                    3 => value!(SoundRate::_44KHZ)
            ),
            switch!(take_bits!(u8, 1),
                    0 => value!(SoundSize::_8Bit)   |
                    1 => value!(SoundSize::_16Bit)
            ),
            switch!(take_bits!(u8, 1),
                    0 => value!(SoundType::Mono)    |
                    1 => value!(SoundType::Stereo)
            )
        ))
    );

    Ok((
        remain,
        AudioTagHeader {
            sound_format,
            sound_rate,
            sound_size,
            sound_type,
        },
    ))
}

#[derive(Debug, Clone, PartialEq)]
pub struct AudioTagBody<'a> {
    pub data: &'a [u8],
}

pub fn audio_tag_body(input: &[u8], size: usize) -> IResult<&[u8], AudioTagBody> {
    if input.len() < size {
        return Err(NomErr::Incomplete(Needed::Size(size)));
    }

    Ok((
        &input[size..],
        AudioTagBody {
            data: &input[0..size],
        },
    ))
}

#[derive(Debug, Clone, PartialEq)]
pub struct AACAudioPacket<'a> {
    /// Only useful when sound format is 10 -- AAC.
    pub packet_type: AACPacketType, // 1 byte.
    pub aac_data: &'a [u8],
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum AACPacketType {
    SequenceHeader, // 0
    Raw,            // 1
}

pub fn aac_audio_packet(input: &[u8], size: usize) -> IResult<&[u8], AACAudioPacket> {
    if input.len() < size {
        return Err(NomErr::Incomplete(Needed::Size(size)));
    }

    if size < 1 {
        return Err(NomErr::Incomplete(Needed::Size(1)));
    }

    let (_, packet_type) = try_parse!(
        input,
        switch!(be_u8,
            0 => value!(AACPacketType::SequenceHeader)  |
            1 => value!(AACPacketType::Raw)
        )
    );

    Ok((
        &input[size..],
        AACAudioPacket {
            packet_type,
            aac_data: &input[1..size],
        },
    ))
}

// ----------------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq)]
pub struct VideoTag<'a> {
    pub header: VideoTagHeader, // 8 bits.
    pub body: VideoTagBody<'a>,
}

pub fn video_tag(input: &[u8], size: usize) -> IResult<&[u8], VideoTag> {
    do_parse!(
        input,
        // VideoTagHeader
        header: apply!(video_tag_header, size)      >>
        // VideoTagBody
        body:   apply!(video_tag_body, size - 1)    >>
        (VideoTag {
            header,
            body,
        })
    )
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct VideoTagHeader {
    pub frame_type: FrameType, // 4 bits.
    pub codec_id: CodecID,     // 4 bits.
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum FrameType {
    Key,             // 1
    Inter,           // 2
    DisposableInter, // 3
    Generated,       // 4
    Command,         // 5
    Unknown,         // Others
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum CodecID {
    //    RGB,          // 0
    //    JPEG,         // 1
    SorensonH263, // 2
    Screen1,      // 3
    VP6,          // 4
    VP6Alpha,     // 5
    Screen2,      // 6
    AVC,          // 7, MPEG-4 Part 10 AVC / H.264
    //    H263,         // 8
    //    MPEG4Part2,   // 9
    Unknown, // Others
}

pub fn video_tag_header(input: &[u8], size: usize) -> IResult<&[u8], VideoTagHeader> {
    if size < 1 {
        return Err(NomErr::Incomplete(Needed::Size(1)));
    }

    let (remain, (frame_type, codec_id)) = try_parse!(
        input,
        bits!(tuple!(
            switch!(take_bits!(u8, 4),
                    1  => value!(FrameType::Key)                |
                    2  => value!(FrameType::Inter)              |
                    3  => value!(FrameType::DisposableInter)    |
                    4  => value!(FrameType::Generated)          |
                    5  => value!(FrameType::Command)            |
                    _  => value!(FrameType::Unknown)
            ),
            switch!(take_bits!(u8, 4),
//                    0 => value!(CodecID::RGB)           |
//                    1 => value!(CodecID::JPEG)          |
                    2 => value!(CodecID::SorensonH263)  |
                    3 => value!(CodecID::Screen1)       |
                    4 => value!(CodecID::VP6)           |
                    5 => value!(CodecID::VP6Alpha)      |
                    6 => value!(CodecID::Screen2)       |
                    7 => value!(CodecID::AVC)           |
//                    8 => value!(CodecID::H263)          |
//                    9 => value!(CodecID::MPEG4Part2)    |
                    _ => value!(CodecID::Unknown)
            )
        ))
    );

    Ok((
        remain,
        VideoTagHeader {
            frame_type,
            codec_id,
        },
    ))
}

#[derive(Debug, Clone, PartialEq)]
pub struct VideoTagBody<'a> {
    pub data: &'a [u8],
}

pub fn video_tag_body(input: &[u8], size: usize) -> IResult<&[u8], VideoTagBody> {
    if input.len() < size {
        return Err(NomErr::Incomplete(Needed::Size(size)));
    }

    Ok((
        &input[size..],
        VideoTagBody {
            data: &input[0..size],
        },
    ))
}

#[derive(Debug, Clone, PartialEq)]
pub struct AVCVideoPacket<'a> {
    /// Only useful when CodecID is 7 -- AVC.
    pub packet_type: AVCPacketType, // 1 byte.
    /// IF packet_type == 1 (NALU)
    ///     composition_time = Composition time offset (in milliseconds)
    /// ELSE
    ///     composition_time = 0
    pub composition_time: i32, // 3 bytes.
    pub avc_data: &'a [u8],
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum AVCPacketType {
    SequenceHeader, // 0
    NALU,           // 1
    EndOfSequence,  // 2
    Unknown,        // Others
}

pub fn avc_video_packet(input: &[u8], size: usize) -> IResult<&[u8], AVCVideoPacket> {
    if input.len() < size {
        return Err(NomErr::Incomplete(Needed::Size(size)));
    }

    if size < 4 {
        return Err(NomErr::Incomplete(Needed::Size(4)));
    }

    let (_, (packet_type, composition_time)) = try_parse!(
        input,
        tuple!(
            switch!(be_u8,
                0 => value!(AVCPacketType::SequenceHeader)  |
                1 => value!(AVCPacketType::NALU)            |
                2 => value!(AVCPacketType::EndOfSequence)   |
                _ => value!(AVCPacketType::Unknown)
                ),
            be_i24
        )
    );

    Ok((
        &input[size..],
        AVCVideoPacket {
            packet_type,
            composition_time,
            avc_data: &input[4..size],
        },
    ))
}

// ----------------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq)]
pub struct ScriptTag<'a> {
    /// Method or object name.
    /// ScriptTagValue.Type = 2 (String)
    pub name: &'a str,
    /// AMF arguments or object properties.
    /// ScriptTagValue.Type = 8 (ECMA array)
    pub value: ScriptDataValue<'a>,
}

static SCRIPT_DATA_VALUE_STRING_TYPE: &'static [u8] = &[0x02];
pub fn script_tag(input: &[u8], _size: usize) -> IResult<&[u8], ScriptTag> {
    do_parse!(
        input,
        // ScriptTagValue.Type = 2 (String)
        tag!(SCRIPT_DATA_VALUE_STRING_TYPE) >>
        // Method or object name.
        name:  script_data_string           >>
        // AMF arguments or object properties.
        // ScriptTagValue.Type = 8 (ECMA array)
        value: script_data_value            >>
        (ScriptTag {
            name,
            value,
        })
    )
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScriptDataValue<'a> {
    Number(f64),                                  // 0
    Boolean(bool),                                // 1
    String(&'a str),                              // 2
    Object(Vec<ScriptDataObjectProperty<'a>>),    // 3
    MovieClip,                                    // 4
    Null,                                         // 5
    Undefined,                                    // 6
    Reference(u16),                               // 7
    ECMAArray(Vec<ScriptDataObjectProperty<'a>>), // 8
    StrictArray(Vec<ScriptDataValue<'a>>),        // 10
    Date(ScriptDataDate),                         // 11
    LongString(&'a str),                          // 12
}

pub fn script_data_value(input: &[u8]) -> IResult<&[u8], ScriptDataValue> {
    //    println!("script_data_value input = {:?}", input);
    switch!(input,
        // Type
        be_u8,
        // Script Data Value
        0  => map!(script_data_number, ScriptDataValue::Number)                     |
        1  => map!(script_data_boolean, |v| ScriptDataValue::Boolean(v != 0))       |
        2  => map!(script_data_string, ScriptDataValue::String)                     |
        3  => map!(script_data_object, ScriptDataValue::Object)                     |
        4  => value!(ScriptDataValue::MovieClip)                                    |
        5  => value!(ScriptDataValue::Null)                                         |
        6  => value!(ScriptDataValue::Undefined)                                    |
        7  => map!(script_data_reference, ScriptDataValue::Reference)               |
        8  => map!(script_data_ecma_array, ScriptDataValue::ECMAArray)              |
        10 => map!(script_data_strict_array, ScriptDataValue::StrictArray)          |
        11 => map!(script_data_date, ScriptDataValue::Date)                         |
        12 => map!(script_data_long_string, ScriptDataValue::LongString)
    )
}

pub fn script_data_number(input: &[u8]) -> IResult<&[u8], f64> {
    //    println!("script_data_number input = {:?}", input);
    be_f64(input)
}

pub fn script_data_boolean(input: &[u8]) -> IResult<&[u8], u8> {
    //    println!("script_data_boolean input = {:?}", input);
    be_u8(input)
}

pub fn script_data_reference(input: &[u8]) -> IResult<&[u8], u16> {
    //    println!("script_data_reference input = {:?}", input);
    be_u16(input)
}

pub fn script_data_string(input: &[u8]) -> IResult<&[u8], &str> {
    //    println!("script_data_string input = {:?}", input);
    map_res!(input, length_bytes!(be_u16), str::from_utf8)
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScriptDataObjectProperty<'a> {
    pub property_name: &'a str,
    pub property_data: ScriptDataValue<'a>,
}

pub fn script_data_object_property(input: &[u8]) -> IResult<&[u8], ScriptDataObjectProperty> {
    //    println!("script_data_object_property input = {:?}", input);
    do_parse!(
        input,
        // Object property name
        name: script_data_string    >>
        // Object property data
        value: script_data_value    >>
        (ScriptDataObjectProperty {
            property_name: name,
            property_data: value,
        })
    )
}

pub fn script_data_object(input: &[u8]) -> IResult<&[u8], Vec<ScriptDataObjectProperty>> {
    //    println!("==============================================================");
    //    println!("script_data_object input = {:?}", input);
    // Script Data Object Property[] and Script Data Object End
    terminated!(
        input,
        many0!(script_data_object_property),
        script_data_object_end_marker
    )
}

static OBJECT_END_MARKER: &'static [u8] = &[0x00, 0x00, 0x09];
pub fn script_data_object_end_marker(input: &[u8]) -> IResult<&[u8], &[u8]> {
    //    println!("script_data_object_end_marker input = {:?}", input);
    tag!(input, OBJECT_END_MARKER)
}

pub fn script_data_ecma_array(input: &[u8]) -> IResult<&[u8], Vec<ScriptDataObjectProperty>> {
    //    println!("==============================================================");
    //    println!("script_data_ecma_array input = {:?}", input);
    // The list contains approximately ECMA Array Length number of items.
    do_parse!(
        input,
        // ECMA Array Length
        _length: be_u32 >>
        // Script Data Object Property[] and Script Data Object End
        value: terminated!(
            many0!(script_data_object_property),
            script_data_object_end_marker
        )               >>
        (value)
    )
}

pub fn script_data_strict_array(input: &[u8]) -> IResult<&[u8], Vec<ScriptDataValue>> {
    //    println!("==============================================================");
    //    println!("script_data_strict_array input = {:?}", input);
    // The list shall contain Strict Array Length number of values.
    // No terminating record follows the list.
    do_parse!(
        input,
        // Strict Array Length
        length: be_u32                                      >>
        // Script Data Value[]
        value: count!(script_data_value, length as usize)   >>
        (value)
    )
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ScriptDataDate {
    /// Number of milliseconds since UNIX_EPOCH.
    // SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis()
    pub date_time: f64,
    /// Local time offset in minutes from UTC.
    /// For time zones located west of Greenwich, this value is a negative number.
    /// Time zones east of Greenwich are positive.
    pub local_date_time_offset: i16,
}

pub fn script_data_date(input: &[u8]) -> IResult<&[u8], ScriptDataDate> {
    //    println!("script_data_date input = {:?}", input);
    do_parse!(
        input,
        // Number of milliseconds since UNIX_EPOCH.
        date_time:              be_f64  >>
        // Local time offset in minutes from UTC.
        local_date_time_offset: be_i16  >>
        (ScriptDataDate {
            date_time,
            local_date_time_offset,
        })
    )
}

pub fn script_data_long_string(input: &[u8]) -> IResult<&[u8], &str> {
    //    println!("script_data_long_string input = {:?}", input);
    map_res!(input, length_bytes!(be_u32), str::from_utf8)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Just use 3 tags of TEST_FLV_FILE:
    // 1. script tag
    // 2. video tag
    // 3. audio tag
    const TEST_FLV_FILE: &'static [u8] = include_bytes!("../assets/test.flv");
    const FLV_FILE_HEADER_LENGTH: usize = 9;
    const PREVIOUS_TAG_SIZE_LENGTH: usize = 4;
    const FLV_TAG_HEADER_LENGTH: usize = 11;
    const AUDIO_TAG_HEADER_LENGTH: usize = 1;
    const VIDEO_TAG_HEADER_LENGTH: usize = 1;

    #[test]
    fn test_parse_flv_file() {
        let flv_file = parse_flv_file(&TEST_FLV_FILE[..]).unwrap().1;
        assert_eq!(
            flv_file.header,
            FLVFileHeader {
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
            flv_file_header(&TEST_FLV_FILE[..end]).unwrap().1
        );
        assert_eq!(
            flv_file_header(&TEST_FLV_FILE[..FLV_FILE_HEADER_LENGTH]),
            Ok((
                &b""[..],
                FLVFileHeader {
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
        let body = flv_file_body(&TEST_FLV_FILE[start..]).unwrap().1;
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
            flv_tag(&TEST_FLV_FILE[start..end]).unwrap().1
        );
        assert_eq!(
            flv_tag(&TEST_FLV_FILE[start..end]),
            Ok((
                &b""[..],
                FLVTag {
                    header: FLVTagHeader {
                        tag_type: FLVTagType::Audio, // 0x08
                        data_size: 7,                // 0x000007
                        timestamp: 0,                // 0x00000000
                        stream_id: 0,                // 0x000000
                    },
                    data: FLVTagData::Audio(AudioTag {
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
                FLVTagHeader {
                    tag_type: FLVTagType::Script, // 0x12
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
                FLVTagHeader {
                    tag_type: FLVTagType::Video, // 0x09
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
                FLVTagHeader {
                    tag_type: FLVTagType::Audio, // 0x08
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
            flv_tag_data(&TEST_FLV_FILE[start..end], FLVTagType::Audio, 7)
                .unwrap()
                .1
        );
        assert_eq!(
            flv_tag_data(&TEST_FLV_FILE[start..end], FLVTagType::Audio, 7),
            Ok((
                &b""[..],
                FLVTagData::Audio(AudioTag {
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

    //    #[test]
    //    fn test_aac_audio_packet() {
    //
    //    }

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

    //    #[test]
    //    fn test_avc_video_packet() {
    //
    //    }

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
