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
#[derive(Debug, Clone, PartialEq)]
pub struct ScriptTag<'a> {
    /// Method or object name.
    /// ScriptTagValue.Type = 2 (String)
    pub name: &'a str,
    /// AMF arguments or object properties.
    /// ScriptTagValue.Type = 8 (ECMAArray)
    pub value: ScriptDataValue<'a>,
}

///
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

///
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

///
pub fn script_data_number(input: &[u8]) -> IResult<&[u8], f64> {
    //    println!("script_data_number input = {:?}", input);
    be_f64(input)
}

///
pub fn script_data_boolean(input: &[u8]) -> IResult<&[u8], u8> {
    //    println!("script_data_boolean input = {:?}", input);
    be_u8(input)
}

///
pub fn script_data_reference(input: &[u8]) -> IResult<&[u8], u16> {
    //    println!("script_data_reference input = {:?}", input);
    be_u16(input)
}

///
pub fn script_data_string(input: &[u8]) -> IResult<&[u8], &str> {
    //    println!("script_data_string input = {:?}", input);
    map_res!(input, length_data!(be_u16), str::from_utf8)
}

/// The `ScriptDataObjectProperty` is the component of `Object` and `ECMAArray`,
/// which are a kind of `ScriptDataValue`.
#[derive(Debug, Clone, PartialEq)]
pub struct ScriptDataObjectProperty<'a> {
    ///
    pub property_name: &'a str,
    ///
    pub property_data: ScriptDataValue<'a>,
}

///
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

///
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

///
pub fn script_data_object_end_marker(input: &[u8]) -> IResult<&[u8], &[u8]> {
    //    println!("script_data_object_end_marker input = {:?}", input);
    tag!(input, OBJECT_END_MARKER)
}

///
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

///
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

/// The `ScriptDataDate` is a kind of `ScriptDataValue`.
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

///
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

///
pub fn script_data_long_string(input: &[u8]) -> IResult<&[u8], &str> {
    //    println!("script_data_long_string input = {:?}", input);
    map_res!(input, length_data!(be_u32), str::from_utf8)
}
