// Copyright 2019-2020 koushiro. Licensed under MIT.

use nom::{be_i24, be_u8, Err as NomErr, IResult, Needed};

/// The tag data part of `video` FLV tag, including `tag data header` and `tag data body`.
#[derive(Debug, Clone, PartialEq)]
pub struct VideoTag<'a> {
    /// The header part of `video` FLV tag.
    pub header: VideoTagHeader, // 8 bits.
    /// The body part of `video` FLV tag.
    pub body: VideoTagBody<'a>,
}

///
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

/// The `tag data header` part of `video` FLV tag data.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct VideoTagHeader {
    /// The frame type of `video` FLV tag, 4 bits.
    pub frame_type: FrameType,
    /// The codec id of `video` FLV tag, 4 bits.
    pub codec_id: CodecID,
}

/// The type of video frame.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum FrameType {
    /// 1, Key frame.
    Key,
    /// 2, Inter frame.
    Inter,
    /// 3, DisposableInter frame.
    DisposableInter,
    /// 4, Generated frame.
    Generated,
    /// 5, Command frame.
    Command,
    /// Unknown frame.
    Unknown,
}

/// The code identifier of video.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum CodecID {
    /// 2, SorensonH263
    SorensonH263,
    /// 3, Screen1
    Screen1,
    /// 4, VP6
    VP6,
    /// 5, VP6Alpha
    VP6Alpha,
    /// 6, Screen2
    Screen2,
    /// 7, MPEG-4 Part 10 AVC / H.264
    AVC,
    /// Unknown codec ID.
    Unknown,
}

///
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
                2 => value!(CodecID::SorensonH263)  |
                3 => value!(CodecID::Screen1)       |
                4 => value!(CodecID::VP6)           |
                5 => value!(CodecID::VP6Alpha)      |
                6 => value!(CodecID::Screen2)       |
                7 => value!(CodecID::AVC)           |
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

/// The `tag data body` part of `video` FLV tag data.
#[derive(Debug, Clone, PartialEq)]
pub struct VideoTagBody<'a> {
    /// The actual `tag data body` of `video` FLV tag data.
    pub data: &'a [u8],
}

///
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

/// The `tag data body` part of `video` FLV tag data whose `CodecID` is 7 -- AVC.
#[derive(Debug, Clone, PartialEq)]
pub struct AvcVideoPacket<'a> {
    /// Only useful when CodecID is 7 -- AVC, 1 byte.
    pub packet_type: AvcPacketType,
    /// The composition time, 3 bytes:
    /// IF packet_type == 1 (NALU)
    ///     composition_time = Composition time offset (in milliseconds)
    /// ELSE
    ///     composition_time = 0
    pub composition_time: i32,
    /// The actual avc data.
    pub avc_data: &'a [u8],
}

/// The type of AVC packet.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum AvcPacketType {
    /// 0, SequenceHeader.
    SequenceHeader,
    /// 1. NALU.
    NALU,
    /// 2, EndOfSequence.
    EndOfSequence,
    /// Unknown
    Unknown,
}

///
pub fn avc_video_packet(input: &[u8], size: usize) -> IResult<&[u8], AvcVideoPacket> {
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
                0 => value!(AvcPacketType::SequenceHeader)  |
                1 => value!(AvcPacketType::NALU)            |
                2 => value!(AvcPacketType::EndOfSequence)   |
                _ => value!(AvcPacketType::Unknown)
            ),
            be_i24
        )
    );

    Ok((
        &input[size..],
        AvcVideoPacket {
            packet_type,
            composition_time,
            avc_data: &input[4..size],
        },
    ))
}
