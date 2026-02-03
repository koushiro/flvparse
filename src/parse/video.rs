use nom::{Err as NomErr, IResult, Needed, bytes::streaming::take, number::streaming::be_u8};

fn be_i24(input: &[u8]) -> IResult<&[u8], i32> {
    let (input, bytes) = take(3usize)(input)?;
    let mut value = (i32::from(bytes[0]) << 16) | (i32::from(bytes[1]) << 8) | i32::from(bytes[2]);
    if (value & 0x0080_0000) != 0 {
        value |= !0x00ff_ffff;
    }
    Ok((input, value))
}

/// The tag data part of `video` FLV tag, including `tag data header` and `tag data body`.
#[derive(Clone, Debug, PartialEq)]
pub struct VideoTag<'a> {
    /// The header part of `video` FLV tag.
    pub header: VideoTagHeader, // 8 bits.
    /// The body part of `video` FLV tag.
    pub body: VideoTagBody<'a>,
}

impl<'a> VideoTag<'a> {
    /// Parse video tag data.
    pub fn parse(input: &'a [u8], size: usize) -> IResult<&'a [u8], VideoTag<'a>> {
        let (input, header) = VideoTagHeader::parse(input, size)?;
        let (input, body) = VideoTagBody::parse(input, size.saturating_sub(1))?;
        Ok((input, VideoTag { header, body }))
    }
}

/// The `tag data header` part of `video` FLV tag data.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VideoTagHeader {
    /// The frame type of `video` FLV tag, 4 bits.
    pub frame_type: FrameType,
    /// The codec id of `video` FLV tag, 4 bits.
    pub codec_id: CodecID,
}

/// The type of video frame.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
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
#[allow(clippy::upper_case_acronyms)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
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

impl VideoTagHeader {
    /// Parse video tag data header.
    pub fn parse(input: &[u8], size: usize) -> IResult<&[u8], VideoTagHeader> {
        if size < 1 {
            return Err(NomErr::Incomplete(Needed::new(1)));
        }

        let (remain, byte) = be_u8(input)?;
        let frame_type = match byte >> 4 {
            1 => FrameType::Key,
            2 => FrameType::Inter,
            3 => FrameType::DisposableInter,
            4 => FrameType::Generated,
            5 => FrameType::Command,
            _ => FrameType::Unknown,
        };

        let codec_id = match byte & 0x0f {
            2 => CodecID::SorensonH263,
            3 => CodecID::Screen1,
            4 => CodecID::VP6,
            5 => CodecID::VP6Alpha,
            6 => CodecID::Screen2,
            7 => CodecID::AVC,
            _ => CodecID::Unknown,
        };

        Ok((remain, VideoTagHeader { frame_type, codec_id }))
    }
}

/// The `tag data body` part of `video` FLV tag data.
#[derive(Clone, Debug, PartialEq)]
pub struct VideoTagBody<'a> {
    /// The actual `tag data body` of `video` FLV tag data.
    pub data: &'a [u8],
}

impl<'a> VideoTagBody<'a> {
    /// Parse video tag data body.
    pub fn parse(input: &'a [u8], size: usize) -> IResult<&'a [u8], VideoTagBody<'a>> {
        if input.len() < size {
            return Err(NomErr::Incomplete(Needed::new(size)));
        }

        Ok((&input[size..], VideoTagBody { data: &input[0..size] }))
    }
}

/// The `tag data body` part of `video` FLV tag data whose `CodecID` is 7 -- AVC.
#[derive(Clone, Debug, PartialEq)]
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
#[allow(clippy::upper_case_acronyms)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
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

/// Parse AVC video packet.
pub fn avc_video_packet(input: &[u8], size: usize) -> IResult<&[u8], AvcVideoPacket<'_>> {
    if input.len() < size {
        return Err(NomErr::Incomplete(Needed::new(size)));
    }

    if size < 4 {
        return Err(NomErr::Incomplete(Needed::new(4)));
    }

    let (remain, packet_type_byte) = be_u8(input)?;
    let packet_type = match packet_type_byte {
        0 => AvcPacketType::SequenceHeader,
        1 => AvcPacketType::NALU,
        2 => AvcPacketType::EndOfSequence,
        _ => AvcPacketType::Unknown,
    };
    let (_, composition_time) = be_i24(remain)?;

    Ok((
        &input[size..],
        AvcVideoPacket { packet_type, composition_time, avc_data: &input[4..size] },
    ))
}
