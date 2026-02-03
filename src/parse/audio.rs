use nom::{
    Err as NomErr, IResult, Needed,
    error::{Error, ErrorKind},
    number::streaming::be_u8,
};

/// The tag data part of `audio` FLV tag, including `tag data header` and `tag data body`.
#[derive(Clone, Debug, PartialEq)]
pub struct AudioTag<'a> {
    /// The header part of `audio` FLV tag.
    pub header: AudioTagHeader, // 8 bits.
    /// The body part of `audio` FLV tag.
    pub body: AudioTagBody<'a>,
}

impl<'a> AudioTag<'a> {
    /// Parse audio tag data.
    pub fn parse(input: &'a [u8], size: usize) -> IResult<&'a [u8], AudioTag<'a>> {
        let (input, header) = AudioTagHeader::parse(input, size)?;
        let (input, body) = AudioTagBody::parse(input, size.saturating_sub(1))?;
        Ok((input, AudioTag { header, body }))
    }
}

/// The `tag data header` part of `audio` FLV tag data.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct AudioTagHeader {
    /// The format of sound, 4 bits.
    pub sound_format: SoundFormat,
    /// The rate of sound, 2 bits.
    pub sound_rate: SoundRate,
    /// The sample size of sound, 1 bit.
    pub sound_size: SoundSize,
    /// The type of sound, 1 bit.
    pub sound_type: SoundType,
}

/// The audio format.
#[allow(clippy::upper_case_acronyms)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SoundFormat {
    /// 0, PcmPlatformEndian
    PcmPlatformEndian,
    /// 1, ADPCM
    ADPCM,
    /// 2, MP3
    MP3,
    /// 3, PcmLittleEndian
    PcmLittleEndian,
    /// 4, Nellymoser16kHzMono
    Nellymoser16kHzMono,
    /// 5, Nellymoser8kHzMono
    Nellymoser8kHzMono,
    /// 6, Nellymoser
    Nellymoser,
    /// 7, PcmALaw
    PcmALaw,
    /// 8, PcmMuLaw
    PcmMuLaw,
    /// 9, Reserved
    Reserved,
    /// 10, MPEG-4 Part3 AAC
    AAC,
    /// 11, Speex
    Speex,
    /// 14, MP3_8kHz
    MP3_8kHz,
    /// 15, DeviceSpecific
    DeviceSpecific,
}

/// The audio sampling rate.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SoundRate {
    /// 0, 5.5 KHz.
    _5_5KHZ,
    /// 1, 11 KHz.
    _11KHZ,
    /// 2, 22 KHz.
    _22KHZ,
    /// 3, 44 KHz.
    _44KHZ,
}

/// The size of each audio sample.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SoundSize {
    /// 0, 8 bit.
    _8Bit,
    /// 1, 16 bit.
    _16Bit,
}

/// The type of audio, including mono and stereo.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SoundType {
    /// 0, Mono sound.
    Mono,
    /// 1, Stereo sound.
    Stereo,
}

impl AudioTagHeader {
    /// Parse audio tag data header.
    pub fn parse(input: &[u8], size: usize) -> IResult<&[u8], AudioTagHeader> {
        if size < 1 {
            return Err(NomErr::Incomplete(Needed::new(1)));
        }

        let (remain, byte) = be_u8(input)?;

        let sound_format = match byte >> 4 {
            0 => SoundFormat::PcmPlatformEndian,
            1 => SoundFormat::ADPCM,
            2 => SoundFormat::MP3,
            3 => SoundFormat::PcmLittleEndian,
            4 => SoundFormat::Nellymoser16kHzMono,
            5 => SoundFormat::Nellymoser8kHzMono,
            6 => SoundFormat::Nellymoser,
            7 => SoundFormat::PcmALaw,
            8 => SoundFormat::PcmMuLaw,
            9 => SoundFormat::Reserved,
            10 => SoundFormat::AAC,
            11 => SoundFormat::Speex,
            14 => SoundFormat::MP3_8kHz,
            15 => SoundFormat::DeviceSpecific,
            _ => SoundFormat::Reserved,
        };

        let sound_rate = match (byte >> 2) & 0b11 {
            0 => SoundRate::_5_5KHZ,
            1 => SoundRate::_11KHZ,
            2 => SoundRate::_22KHZ,
            3 => SoundRate::_44KHZ,
            _ => SoundRate::_5_5KHZ,
        };

        let sound_size = match (byte >> 1) & 0b1 {
            0 => SoundSize::_8Bit,
            1 => SoundSize::_16Bit,
            _ => SoundSize::_8Bit,
        };

        let sound_type = match byte & 0b1 {
            0 => SoundType::Mono,
            1 => SoundType::Stereo,
            _ => SoundType::Mono,
        };

        Ok((remain, AudioTagHeader { sound_format, sound_rate, sound_size, sound_type }))
    }
}

/// The `tag data body` part of `audio` FLV tag data.
#[derive(Clone, Debug, PartialEq)]
pub struct AudioTagBody<'a> {
    /// The actual `tag data body` of `audio` FLV tag data.
    pub data: &'a [u8],
}

impl<'a> AudioTagBody<'a> {
    /// Parse audio tag data body.
    pub fn parse(input: &'a [u8], size: usize) -> IResult<&'a [u8], AudioTagBody<'a>> {
        if input.len() < size {
            return Err(NomErr::Incomplete(Needed::new(size)));
        }

        Ok((&input[size..], AudioTagBody { data: &input[0..size] }))
    }
}

/// The `tag data body` part of `audio` FLV tag data whose `SoundFormat` is 10 -- AAC.
#[derive(Clone, Debug, PartialEq)]
pub struct AACAudioPacket<'a> {
    /// Only useful when sound format is 10 -- AAC, 1 byte.
    pub packet_type: AACPacketType,
    /// The actual AAC data.
    pub aac_data: &'a [u8],
}

/// The type of AAC packet.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum AACPacketType {
    /// 0, SequenceHeader.
    SequenceHeader,
    /// 1, Raw.
    Raw,
}

/// Parse AAC audio packet.
pub fn aac_audio_packet(input: &[u8], size: usize) -> IResult<&[u8], AACAudioPacket<'_>> {
    if input.len() < size {
        return Err(NomErr::Incomplete(Needed::new(size)));
    }

    if size < 1 {
        return Err(NomErr::Incomplete(Needed::new(1)));
    }

    let (remain, packet_type_byte) = be_u8(input)?;
    let packet_type = match packet_type_byte {
        0 => AACPacketType::SequenceHeader,
        1 => AACPacketType::Raw,
        _ => return Err(NomErr::Error(Error::new(remain, ErrorKind::Switch))),
    };

    Ok((&input[size..], AACAudioPacket { packet_type, aac_data: &input[1..size] }))
}
