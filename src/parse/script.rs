// Copyright 2019-2020 koushiro. Licensed under MIT.

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::vec::Vec;
use core::str;

use nom::{
    number::streaming::{be_f64, be_i16, be_u16, be_u32, be_u8},
    IResult,
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
        do_parse!(
            input,
            // ScriptTagValue.Type = 2 (String)
            tag!(SCRIPT_DATA_VALUE_STRING_TYPE) >>
            // Method or object name.
            name: call!(ScriptDataValue::parse_string) >>
            // AMF arguments or object properties.
            // ScriptTagValue.Type = 8 (ECMA array)
            value: call!(ScriptDataValue::parse) >>

            (ScriptTag { name, value })
        )
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
    /// Parse script data value.
    pub fn parse(input: &'a [u8]) -> IResult<&'a [u8], ScriptDataValue<'a>> {
        switch!(input,
            // parse script value type
            be_u8,
            // parse script data value
            0  => map!(Self::parse_number, ScriptDataValue::Number)               |
            1  => map!(Self::parse_boolean, |v| ScriptDataValue::Boolean(v != 0)) |
            2  => map!(Self::parse_string, ScriptDataValue::String)               |
            3  => map!(Self::parse_object, ScriptDataValue::Object)               |
            4  => value!(ScriptDataValue::MovieClip)                              |
            5  => value!(ScriptDataValue::Null)                                   |
            6  => value!(ScriptDataValue::Undefined)                              |
            7  => map!(Self::parse_reference, ScriptDataValue::Reference)         |
            8  => map!(Self::parse_ecma_array, ScriptDataValue::ECMAArray)        |
            10 => map!(Self::parse_strict_array, ScriptDataValue::StrictArray)    |
            11 => map!(Self::parse_date, ScriptDataValue::Date)                   |
            12 => map!(Self::parse_long_string, ScriptDataValue::LongString)
        )
    }

    /// Parse script number value.
    pub fn parse_number(input: &[u8]) -> IResult<&[u8], f64> {
        be_f64(input)
    }

    /// Parse script boolean value.
    pub fn parse_boolean(input: &[u8]) -> IResult<&[u8], u8> {
        be_u8(input)
    }

    /// Parse script string value.
    pub fn parse_string(input: &[u8]) -> IResult<&[u8], &str> {
        map_res!(input, length_data!(be_u16), str::from_utf8)
    }

    /// Parse script object value.
    pub fn parse_object(input: &'a [u8]) -> IResult<&'a [u8], Vec<ScriptDataObjectProperty<'a>>> {
        terminated!(
            input,
            // parse object properties
            many0!(Self::parse_object_property),
            // parse object end marker
            call!(Self::parse_object_end_marker)
        )
    }

    /// Parse script object property.
    fn parse_object_property(input: &'a [u8]) -> IResult<&'a [u8], ScriptDataObjectProperty<'a>> {
        do_parse!(
            input,
            // parse object property name
            name: call!(Self::parse_string) >>
            // parse object property value
            value: call!(Self::parse) >>

            (ScriptDataObjectProperty { name, value })
        )
    }

    /// Parse script object end marker.
    fn parse_object_end_marker(input: &[u8]) -> IResult<&[u8], &[u8]> {
        tag!(input, OBJECT_END_MARKER)
    }

    /// Parse script reference value.
    pub fn parse_reference(input: &[u8]) -> IResult<&[u8], u16> {
        be_u16(input)
    }

    /// Parse script ECMA array value.
    pub fn parse_ecma_array(
        input: &'a [u8],
    ) -> IResult<&'a [u8], Vec<ScriptDataObjectProperty<'a>>> {
        // The list contains approximately ECMA Array Length number of items.
        do_parse!(
            input,
            // parse ECMA array length
            _length: be_u32 >>
            // parse object Properties and Object End marker
            object: call!(Self::parse_object) >>

            (object)
        )
    }

    /// Parse script strict array value.
    pub fn parse_strict_array(input: &'a [u8]) -> IResult<&'a [u8], Vec<ScriptDataValue<'a>>> {
        // The list shall contain Strict Array Length number of values.
        // No terminating record follows the list.
        do_parse!(
            input,
            // parse strict array length
            length: be_u32 >>
            // parse values
            value: count!(call!(Self::parse), length as usize) >>

            (value)
        )
    }

    /// Parse script date value.
    pub fn parse_date(input: &[u8]) -> IResult<&[u8], ScriptDataDate> {
        do_parse!(
            input,
            // Number of milliseconds since UNIX_EPOCH.
            date_time: be_f64 >>
            // Local time offset in minutes from UTC.
            local_date_time_offset: be_i16 >>

            (ScriptDataDate { date_time, local_date_time_offset })
        )
    }

    /// Parse script long string value.
    pub fn parse_long_string(input: &[u8]) -> IResult<&[u8], &str> {
        map_res!(input, length_data!(be_u32), str::from_utf8)
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
