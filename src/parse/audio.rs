// Copyright 2019-2020 koushiro. Licensed under MIT.

use nom::{be_u8, Err as NomErr, IResult, Needed};

/// The tag data part of `audio` FLV tag, including `tag data header` and `tag data body`.
#[derive(Debug, Clone, PartialEq)]
pub struct AudioTag<'a> {
    /// The header part of `audio` FLV tag.
    pub header: AudioTagHeader, // 8 bits.
    /// The body part of `audio` FLV tag.
    pub body: AudioTagBody<'a>,
}

///
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

/// The `tag data header` part of `audio` FLV tag data.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
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
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
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
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
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
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SoundSize {
    /// 0, 8 bit.
    _8Bit,
    /// 1, 16 bit.
    _16Bit,
}

/// The type of audio, including mono and stereo.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SoundType {
    /// 0, Mono sound.
    Mono,
    /// 1, Stereo sound.
    Stereo,
}

///
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

/// The `tag data body` part of `audio` FLV tag data.
#[derive(Debug, Clone, PartialEq)]
pub struct AudioTagBody<'a> {
    /// The actual `tag data body` of `audio` FLV tag data.
    pub data: &'a [u8],
}

///
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

/// The `tag data body` part of `audio` FLV tag data whose `SoundFormat` is 10 -- AAC.
#[derive(Debug, Clone, PartialEq)]
pub struct AACAudioPacket<'a> {
    /// Only useful when sound format is 10 -- AAC, 1 byte.
    pub packet_type: AACPacketType,
    /// The actual AAC data.
    pub aac_data: &'a [u8],
}

/// The type of AAC packet.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum AACPacketType {
    /// 0, SequenceHeader.
    SequenceHeader,
    /// 1, Raw.
    Raw,
}

///
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
