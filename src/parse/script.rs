#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
use core::str;

use nom::{
    Err as NomErr,
    IResult,
    Parser,
    bytes::streaming::tag,
    combinator::map_res,
    error::{Error, ErrorKind},
    multi::{count, length_data, many0},
    number::streaming::{be_f64, be_i16, be_u8, be_u16, be_u32},
    sequence::terminated,
};

const SCRIPT_DATA_VALUE_STRING_TYPE: [u8; 1] = [0x02];
const OBJECT_END_MARKER: [u8; 3] = [0x00, 0x00, 0x09];

/// The tag data part of `script` FLV tag, including `name` and `value`.
/// The `name` is a `ScriptDataValue` enum whose type is `String`.
/// The `value` is a `ScriptDataValue` enum whose type is `ECMAArray`.
#[derive(Clone, Debug, PartialEq)]
pub struct ScriptTag<'a> {
    /// Method or object name.
    /// ScriptTagValue.Type = 2 (String)
    pub name: &'a str,
    /// AMF arguments or object properties.
    /// ScriptTagValue.Type = 8 (ECMAArray)
    pub value: ScriptDataValue<'a>,
}

impl<'a> ScriptTag<'a> {
    /// Parse script tag data.
    pub fn parse(input: &'a [u8], _size: usize) -> IResult<&'a [u8], ScriptTag<'a>> {
        let (input, _) = tag(SCRIPT_DATA_VALUE_STRING_TYPE.as_slice())(input)?;
        let (input, name) = ScriptDataValue::parse_string(input)?;
        let (input, value) = ScriptDataValue::parse(input)?;
        Ok((input, ScriptTag { name, value }))
    }
}

/// The `ScriptDataValue` enum.
#[derive(Debug, Clone, PartialEq)]
pub enum ScriptDataValue<'a> {
    /// 0, Number value.
    Number(f64),
    /// 1, Boolean value.
    Boolean(bool),
    /// 2, String value.
    String(&'a str),
    /// 3, Object value.
    Object(Vec<ScriptDataObjectProperty<'a>>),
    /// 4, MovieClip value.
    MovieClip,
    /// 5, Null value.
    Null,
    /// 6, Undefined value.
    Undefined,
    /// 7, Reference value.
    Reference(u16),
    /// 8, ECMA Array value.
    ECMAArray(Vec<ScriptDataObjectProperty<'a>>),
    /// 10, Strict Array value.
    StrictArray(Vec<ScriptDataValue<'a>>),
    /// 11, Date value.
    Date(ScriptDataDate),
    /// 12, Long String value.
    LongString(&'a str),
}

impl<'a> ScriptDataValue<'a> {
    /// Parse script tag data value.
    pub fn parse(input: &'a [u8]) -> IResult<&'a [u8], ScriptDataValue<'a>> {
        let original_input = input;
        let (input, value_type) = be_u8(input)?;
        match value_type {
            0 => {
                let (input, number) = Self::parse_number(input)?;
                Ok((input, ScriptDataValue::Number(number)))
            }
            1 => {
                let (input, v) = Self::parse_boolean(input)?;
                Ok((input, ScriptDataValue::Boolean(v != 0)))
            }
            2 => {
                let (input, s) = Self::parse_string(input)?;
                Ok((input, ScriptDataValue::String(s)))
            }
            3 => {
                let (input, object) = Self::parse_object(input)?;
                Ok((input, ScriptDataValue::Object(object)))
            }
            4 => Ok((input, ScriptDataValue::MovieClip)),
            5 => Ok((input, ScriptDataValue::Null)),
            6 => Ok((input, ScriptDataValue::Undefined)),
            7 => {
                let (input, reference) = Self::parse_reference(input)?;
                Ok((input, ScriptDataValue::Reference(reference)))
            }
            8 => {
                let (input, array) = Self::parse_ecma_array(input)?;
                Ok((input, ScriptDataValue::ECMAArray(array)))
            }
            10 => {
                let (input, array) = Self::parse_strict_array(input)?;
                Ok((input, ScriptDataValue::StrictArray(array)))
            }
            11 => {
                let (input, date) = Self::parse_date(input)?;
                Ok((input, ScriptDataValue::Date(date)))
            }
            12 => {
                let (input, s) = Self::parse_long_string(input)?;
                Ok((input, ScriptDataValue::LongString(s)))
            }
            _ => Err(NomErr::Error(Error::new(original_input, ErrorKind::Switch))),
        }
    }

    /// Parse script tag data number value.
    pub fn parse_number(input: &[u8]) -> IResult<&[u8], f64> {
        be_f64(input)
    }

    /// Parse script tag data boolean value.
    pub fn parse_boolean(input: &[u8]) -> IResult<&[u8], u8> {
        be_u8(input)
    }

    /// Parse script tag data string value.
    pub fn parse_string(input: &[u8]) -> IResult<&[u8], &str> {
        map_res(length_data(be_u16), str::from_utf8).parse(input)
    }

    /// Parse script tag data object value.
    pub fn parse_object(input: &'a [u8]) -> IResult<&'a [u8], Vec<ScriptDataObjectProperty<'a>>> {
        terminated(many0(Self::parse_object_property), Self::parse_object_end_marker).parse(input)
    }

    /// Parse script tag data object property.
    fn parse_object_property(input: &'a [u8]) -> IResult<&'a [u8], ScriptDataObjectProperty<'a>> {
        if input.starts_with(&OBJECT_END_MARKER) {
            return Err(NomErr::Error(Error::new(input, ErrorKind::Tag)));
        }
        let (input, (name, value)) = (Self::parse_string, Self::parse).parse(input)?;
        Ok((input, ScriptDataObjectProperty { name, value }))
    }

    /// Parse script tag data object end marker.
    fn parse_object_end_marker(input: &[u8]) -> IResult<&[u8], &[u8]> {
        tag(OBJECT_END_MARKER.as_slice())(input)
    }

    /// Parse script tag data reference value.
    pub fn parse_reference(input: &[u8]) -> IResult<&[u8], u16> {
        be_u16(input)
    }

    /// Parse script tag data ECMA array value.
    pub fn parse_ecma_array(
        input: &'a [u8],
    ) -> IResult<&'a [u8], Vec<ScriptDataObjectProperty<'a>>> {
        // The list contains approximately ECMA Array Length number of items.
        let (input, _) = be_u32(input)?;
        Self::parse_object(input)
    }

    /// Parse script tag data strict array value.
    pub fn parse_strict_array(input: &'a [u8]) -> IResult<&'a [u8], Vec<ScriptDataValue<'a>>> {
        // The list shall contain Strict Array Length number of values.
        // No terminating record follows the list.
        let (input, length) = be_u32(input)?;
        count(Self::parse, length as usize).parse(input)
    }

    /// Parse script tag data date value.
    pub fn parse_date(input: &[u8]) -> IResult<&[u8], ScriptDataDate> {
        let (input, (date_time, local_date_time_offset)) = (be_f64, be_i16).parse(input)?;
        Ok((input, ScriptDataDate { date_time, local_date_time_offset }))
    }

    /// Parse script tag data long string value.
    pub fn parse_long_string(input: &[u8]) -> IResult<&[u8], &str> {
        map_res(length_data(be_u32), str::from_utf8).parse(input)
    }
}

/// The `ScriptDataObjectProperty` is the component of `Object` and `ECMAArray`,
/// which are a kind of `ScriptDataValue`.
#[derive(Clone, Debug, PartialEq)]
pub struct ScriptDataObjectProperty<'a> {
    /// Object property name.
    pub name: &'a str,
    /// Object property value.
    pub value: ScriptDataValue<'a>,
}

/// The `ScriptDataDate` is a kind of `ScriptDataValue`.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ScriptDataDate {
    /// Number of milliseconds since UNIX_EPOCH.
    // SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis()
    pub date_time: f64,
    /// Local time offset in minutes from UTC.
    /// For time zones located west of Greenwich, this value is a negative number.
    /// Time zones east of Greenwich are positive.
    pub local_date_time_offset: i16,
}
