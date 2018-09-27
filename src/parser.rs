/// https://www.adobe.com/content/dam/acom/en/devnet/flv/video_file_format_spec_v10_1.pdf
use std::str;

use nom::{
    be_u8, be_u16, be_i16, be_u24, be_i24, be_u32, be_f64,
    IResult, Needed,
    Err as NomErr
};

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
    pub has_audio: bool,
    pub has_video: bool,
    /// The length of this header in bytes, usually has a value of 9 for FLV version 1.
    pub data_offset: u32,
}

static FLV_HEADER_SIGNATURE: &'static [u8] = &[0x46, 0x4c, 0x56];
pub fn flv_file_header(input: &[u8]) -> IResult<&[u8], FLVFileHeader> {
    do_parse!(
        input,
        tag!(FLV_HEADER_SIGNATURE)  >>
        version:     be_u8          >>
        flags:       be_u8          >>
        data_offset: be_u32         >>
        (FLVFileHeader {
            signature: [0x46, 0x4c, 0x56],
            version,
            has_audio: flags & 4 == 4,
            has_video: flags & 1 == 1,
            data_offset,
        })
    )
}

#[derive(Debug, Clone, PartialEq)]
pub struct FLVFileBody<'a> {
    /// The size of the first tag is always 0.
    pub first_tag_size: u32,
    /// FLV Tag and the size of the tag.
    pub tags: Vec<(FLVTag<'a>, u32)>,
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
        header: call!(flv_tag_header)                           >>
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
        tag_type: switch!(be_u8,
            8  => value!(FLVTagType::Audio) |
            9  => value!(FLVTagType::Video) |
            18 => value!(FLVTagType::Script)
        )                           >>
        data_size:          be_u24  >>
        timestamp:          be_u24  >>
        timestamp_extended: be_u8   >>
        stream_id:          be_u24  >>
        (FLVTagHeader {
            tag_type,
            data_size,
            timestamp: ((timestamp_extended as u32) << 24) + timestamp,
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

pub fn flv_tag_data(input: &[u8], tag_type: FLVTagType, size: usize)
    -> IResult<&[u8], FLVTagData>
{
    match tag_type {
        FLVTagType::Audio  => map!(
            input,
            apply!(audio_tag, size),
            |data| FLVTagData::Audio(data)
        ),
        FLVTagType::Video  => map!(
            input,
            apply!(video_tag, size),
            |data| FLVTagData::Video(data)
        ),
        FLVTagType::Script => map!(
            input,
            apply!(script_tag, size),
            |data| FLVTagData::Script(data)
        )
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
        header: apply!(audio_tag_header, size)      >>
        body:   apply!(audio_tag_body, size - 1)    >>
        (AudioTag {
            header,
            body,
        })
    )
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct AudioTagHeader {
    pub sound_format: SoundFormat,  // 4 bits.
    pub sound_rate: SoundRate,      // 2 bits.
    pub sound_size: SoundSize,      // 1 bit.
    pub sound_type: SoundType,      // 1 bit.
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SoundFormat {
    PcmPlatformEndian,      // 0
    ADPCM,                  // 1
    MP3,                    // 2
    PcmLittleEndian,        // 3
    Nellymoser16kHzMono,    // 4
    Nellymoser8kHzMono,     // 5
    Nellymoser,             // 6
    PcmALaw,                // 7
    PcmMuLaw,               // 8
    Reserved,               // 9
    AAC,                    // 10
    Speex,                  // 11
    MP3_8kHz,               // 14
    DeviceSpecific,         // 15
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SoundRate {
    _5_5KHZ,    // 0
    _11KHZ,     // 1
    _22KHZ,     // 2
    _44KHZ,     // 3
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SoundSize {
    _8Bits,     // 0
    _16Bits,    // 1
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
        bits!(
            tuple!(
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
                    0 => value!(SoundSize::_8Bits)  |
                    1 => value!(SoundSize::_16Bits)
                ),
                switch!(take_bits!(u8, 1),
                    0 => value!(SoundType::Mono)    |
                    1 => value!(SoundType::Stereo)
                )
            )
        )
    );

    Ok((remain, AudioTagHeader {
        sound_format,
        sound_rate,
        sound_size,
        sound_type,
    }))
}

#[derive(Debug, Clone, PartialEq)]
pub struct AudioTagBody<'a> {
    pub data: &'a [u8],
}

pub fn audio_tag_body(input: &[u8], size: usize) -> IResult<&[u8], AudioTagBody> {
    if input.len() < size {
        return Err(NomErr::Incomplete(Needed::Size(size)));
    }

    Ok((&input[size..], AudioTagBody {
        data: &input[0..size],
    }))
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

    Ok((&input[size..], AACAudioPacket {
        packet_type,
        aac_data: &input[1..size],
    }))
}

// ----------------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq)]
pub struct VideoTag<'a> {
    pub header: VideoTagHeader,    // 8 bits.
    pub body: VideoTagBody<'a>,
}

pub fn video_tag(input: &[u8], size: usize) -> IResult<&[u8], VideoTag> {
    do_parse!(
        input,
        header: apply!(video_tag_header, size)  >>
        body:   apply!(video_tag_body, size)    >>
        (VideoTag {
            header,
            body,
        })
    )
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct VideoTagHeader {
    pub frame_type: FrameType,  // 4 bits.
    pub codec_id: CodecID,      // 4 bits.
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum FrameType {
    Key,                // 1
    Inter,              // 2
    DisposableInter,    // 3
    Generated,          // 4
    Command,            // 5
    Unknown,            // Others
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum CodecID {
//    RGB,            // 0
//    JPEG,           // 1
    SorensonH263,   // 2
    Screen1,        // 3
    VP6,            // 4
    VP6Alpha,       // 5
    Screen2,        // 6
    AVC,            // 7
//    H263,           // 8
//    MPEG4Part2,     // 9
    Unknown,        // Others
}

pub fn video_tag_header(input: &[u8], size: usize) -> IResult<&[u8], VideoTagHeader> {
    if size < 1 {
        return Err(NomErr::Incomplete(Needed::Size(1)));
    }

    let (remain, (frame_type, codec_id)) = try_parse!(
        input,
        bits!(
            tuple!(
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
            )
        )
    );

    Ok((remain, VideoTagHeader {
        frame_type,
        codec_id,
    }))
}

#[derive(Debug, Clone, PartialEq)]
pub struct VideoTagBody<'a> {
    pub data: &'a [u8],
}

pub fn video_tag_body(input: &[u8], size: usize) -> IResult<&[u8], VideoTagBody> {
    if input.len() < size {
        return Err(NomErr::Incomplete(Needed::Size(size)));
    }

    Ok((&input[size..], VideoTagBody {
        data: &input[0..size],
    }))
}

#[derive(Debug, Clone, PartialEq)]
pub struct AVCVideoPacket<'a> {
    /// Only useful when CodecID is 7 -- AVC.
    pub packet_type: AVCPacketType, // 1 byte.
    /// IF packet_type == 1 (NALU)
    ///     composition_time = Composition time offset (in milliseconds)
    /// ELSE
    ///     composition_time = 0
    pub composition_time: i32,      // 3 bytes.
    pub avc_data: &'a [u8],
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum AVCPacketType {
    SequenceHeader,     // 0
    NALU,               // 1
    EndOfSequence,      // 2
    Unknown,            // Others
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

    Ok((&input[size..], AVCVideoPacket {
        packet_type,
        composition_time,
        avc_data: &input[4..size],
    }))
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

static SCRIPT_DATA_VALUE_STRING_TYPE: &'static [u8] = &[2];
pub fn script_tag(input: &[u8], _size: usize) -> IResult<&[u8], ScriptTag> {
    do_parse!(
        input,
        // Method or object name
        // ScriptTagValue.Type = 2 (String)
        tag!(SCRIPT_DATA_VALUE_STRING_TYPE) >>
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
    Number(f64),                                    // 0
    Boolean(bool),                                  // 1
    String(&'a str),                                // 2
    Object(Vec<ScriptDataObjectProperty<'a>>),      // 3
    MovieClip,                                      // 4
    Null,                                           // 5
    Undefined,                                      // 6
    Reference(u16),                                 // 7
    ECMAArray(Vec<ScriptDataObjectProperty<'a>>),   // 8
    ObjectEndMarker(&'a [u8]),                      // 9
    StrictArray(Vec<ScriptDataValue<'a>>),          // 10
    Date(ScriptDataDate),                           // 11
    LongString(&'a str),                            // 12
}

pub fn script_data_value(input: &[u8]) -> IResult<&[u8], ScriptDataValue> {
    switch!(input,
        // Type
        be_u8,
        // Script Data Value
        0  => map!(be_f64, |v| ScriptDataValue::Number(v))                                  |
        1  => map!(be_u8, |v| ScriptDataValue::Boolean(v != 0))                             |
        2  => map!(script_data_string, |v| ScriptDataValue::String(v))                      |
        3  => map!(script_data_object, |v| ScriptDataValue::Object(v))                      |
        4  => value!(ScriptDataValue::MovieClip)                                            |
        5  => value!(ScriptDataValue::Null)                                                 |
        6  => value!(ScriptDataValue::Undefined)                                            |
        7  => map!(be_u16, |v| ScriptDataValue::Reference(v))                               |
        8  => map!(script_data_ecma_array, |v| ScriptDataValue::ECMAArray(v))               |
        9  => map!(script_data_object_end_marker, |v| ScriptDataValue::ObjectEndMarker(v))  |
        10 => map!(script_data_strict_array, |v| ScriptDataValue::StrictArray(v))           |
        11 => map!(script_data_date, |v| ScriptDataValue::Date(v))                          |
        12 => map!(script_data_long_string, |v| ScriptDataValue::LongString(v))
    )
}

pub fn script_data_string(input: &[u8]) -> IResult<&[u8], &str> {
    map_res!(input, length_bytes!(be_u16), str::from_utf8)
}

pub fn script_data_long_string(input: &[u8]) -> IResult<&[u8], &str> {
    map_res!(input, length_bytes!(be_u32), str::from_utf8)
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
    do_parse!(
        input,
        date_time:              be_f64  >>
        local_date_time_offset: be_i16  >>
        (ScriptDataDate {
            date_time,
            local_date_time_offset,
        })
    )
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScriptDataObjectProperty<'a> {
    pub property_name: &'a str,
    pub property_data: ScriptDataValue<'a>,
}

pub fn script_data_object_property(input: &[u8]) -> IResult<&[u8], ScriptDataObjectProperty> {
    do_parse!(
        input,
        name: script_data_string    >>
        value: script_data_value    >>
        (ScriptDataObjectProperty {
            property_name: name,
            property_data: value,
        })
    )
}

static OBJECT_END_MARKER: &'static [u8] = &[0, 0, 9];
pub fn script_data_object_end_marker(input: &[u8]) -> IResult<&[u8], &[u8]> {
    tag!(input, OBJECT_END_MARKER)
}

pub fn script_data_object(input: &[u8]) -> IResult<&[u8], Vec<ScriptDataObjectProperty>> {
    // Script Data Object Property[] and Script Data Object End
    terminated!(
        input,
        many0!(
            call!(script_data_object_property)
        ),
        script_data_object_end_marker
    )
}

pub fn script_data_ecma_array(input: &[u8]) -> IResult<&[u8], Vec<ScriptDataObjectProperty>> {
    // The list contains approximately ECMA Array Length number of items.
    do_parse!(
        input,
        // ECMA Array Length
        _length: be_u32             >>
        // Script Data Object Property[] and Script Data Object End
        value:  script_data_object  >>
        (value)
    )
}

pub fn script_data_strict_array(input: &[u8]) -> IResult<&[u8], Vec<ScriptDataValue>> {
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

#[cfg(test)]
mod tests {
    use super::*;


}