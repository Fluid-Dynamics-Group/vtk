//! reading and parsing xml VTK files
//!
//! most of the time you will not need to interact with this file,
//! instead derive `ParseDataArray`
use super::data::{LocationSpans, VtkData};
use super::ParseDataArray;
use crate::utils;
use crate::Error;

use std::io::Read;

use nom::bytes::complete::{tag, take, take_till, take_until};
use nom::IResult;

use std::fmt;

type NomErr<'a> = nom::Err<nom::error::Error<&'a [u8]>>;

/// An error caused from parsing the vtk files
#[derive(Debug, thiserror::Error)]
pub struct ParseError {
    nom_reason: Vec<u8>,
    nom_code: nom::error::ErrorKind,
    extra_info: &'static str,
}

impl ParseError {
    pub fn from_nom(x: NomErr, extra_info: &'static str) -> Self {
        let error = match x {
            nom::Err::Incomplete(_) => unreachable!(),
            nom::Err::Error(e) => e,
            nom::Err::Failure(e) => e,
        };
        Self {
            nom_reason: error.input.to_vec(),
            nom_code: error.code,
            extra_info,
        }
    }
}
impl<'a> From<NomErr<'a>> for ParseError {
    fn from(x: NomErr<'a>) -> Self {
        ParseError::from_nom(x, "Caused by From Impl")
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match String::from_utf8(self.nom_reason.clone()) {
            Ok(string_representation) => {
                write!(
                    f,
                    "reason:{} \tnom_reason:{} \t errorcode:{:?}",
                    self.extra_info, string_representation, self.nom_code
                )
            }
            Err(_) => {
                write!(
                    f,
                    "reason:{} \t <nom reason omitted> \t errorcode:{:?} (could not convert nom bytes to string- fallback)",
                    self.extra_info, self.nom_code
                )
            }
        }
    }
}

/// read in and parse an entire vtk file for a given path
pub fn read_and_parse<D: ParseDataArray>(path: &std::path::Path) -> Result<VtkData<D>, Error> {
    let mut file = std::fs::File::open(path)?;
    let mut buffer = Vec::with_capacity(1024 * 1024 * 3);
    file.read_to_end(&mut buffer)?;

    parse_xml_document(&buffer)
}

pub(crate) fn parse_xml_document<D: ParseDataArray>(i: &[u8]) -> Result<VtkData<D>, Error> {
    let (rest_of_document, spans) = find_extent(i).map_err(|e: NomErr| {
        ParseError::from_nom(
            e,
            "Error in parsing find_extent for the WholeExtent span information",
        )
    })?;

    let (rest_of_document, locations) =
        parse_locations(rest_of_document, &spans).map_err(|e: NomErr| {
            ParseError::from_nom(e, "Error in parsing the location data of the document")
        })?;

    let (data, locations) = D::parse_dataarrays(rest_of_document, &spans, locations)?;

    Ok(VtkData {
        spans,
        locations,
        data,
    })
}

#[allow(dead_code)]
fn print_n_chars(i: &[u8], chars: usize) {
    if i.len() > chars {
        let slice = i.get(0..chars).unwrap();
        match std::str::from_utf8(slice) {
            Ok(string_slice) => println!("{}", string_slice),
            Err(_e) => println!("-- could not parse bytes as string"),
        }
    } else {
        println!("ommitting print since string is too short")
    }
}

pub(crate) fn find_extent(i: &[u8]) -> IResult<&[u8], LocationSpans> {
    let (start_extent, _xml_header_info) = take_until("WholeExtent")(i)?;
    let (extent_string_start, _whole_extent_header) = tag("WholeExtent=\"")(start_extent)?;
    let (rest_of_document, extent_string) = take_till(|c| c == b'\"')(extent_string_start)?;

    let spans = LocationSpans::new(std::str::from_utf8(extent_string).unwrap());

    Ok((rest_of_document, spans))
}

pub(crate) fn parse_locations<'a>(
    i: &'a [u8],
    span_info: &LocationSpans,
) -> IResult<&'a [u8], LocationsPartial> {
    let (rest, x) = parse_dataarray_or_lazy(i, b"X", span_info.x_len())?;
    let (rest, y) = parse_dataarray_or_lazy(rest, b"Y", span_info.y_len())?;
    let (rest, z) = parse_dataarray_or_lazy(rest, b"Z", span_info.z_len())?;

    let locations = LocationsPartial { x, y, z };

    Ok((rest, locations))
}

/// Parse a data array (if its inline) or return the offset in the appended section
pub fn parse_dataarray_or_lazy<'a>(
    xml_bytes: &'a [u8],
    expected_data: &[u8],
    size_hint: usize,
) -> IResult<&'a [u8], PartialDataArray> {
    let (mut rest, header) = read_dataarray_header(xml_bytes, expected_data)?;
    let lazy_array = match header {
        DataArrayHeader::AppendedBinary { offset } => PartialDataArray::AppendedBinary { offset },
        DataArrayHeader::InlineAscii => {
            let (after_dataarray, parsed_data) = parse_ascii_inner_dataarray(rest, size_hint)?;
            rest = after_dataarray;
            PartialDataArray::Parsed(parsed_data)
        }
        DataArrayHeader::InlineBase64 => {
            let (after_dataarray, parsed_data) = parse_base64_inner_dataarray(rest, size_hint)?;
            rest = after_dataarray;
            PartialDataArray::Parsed(parsed_data)
        }
    };

    Ok((rest, lazy_array))
}

/// all of the location data - either containing already parsed informatoin
/// or references to the offsets in the appended binary section
pub struct LocationsPartial {
    pub x: PartialDataArray,
    pub y: PartialDataArray,
    pub z: PartialDataArray,
}

/// read through a DataArray header and consume up to the ending `>` character of the
/// header.
///
/// ### `xml_bytes`
///
/// is the string slice that starts with the after the data array header information. This
/// is most easily accomplished by a call to `read_dataarray_header`. This bytes slice
/// may contain more information after the dataarray, which is returned from this function.
///
/// ### `expected_data`
///
/// byte string of the expected `Name` attribute for this `DataArray`
///
/// ## Returns
///
/// A tuple of the remaining data in the string (not parsed) and the floating point data that
/// was contained in the DataArray
/// Currently expects the attributes of the header to be in this format:
///
/// ```ignore
/// <DataArray name="name here" format="format here" offset="offset, if appended format"> ...
/// ```
///
/// also assumes NumberOfComponents=1 and type=Float64
pub fn read_dataarray_header<'a>(
    xml_bytes: &'a [u8],
    expected_data: &[u8],
) -> IResult<&'a [u8], DataArrayHeader> {
    let (name_start, _) = take_until_consume(xml_bytes, b"Name=")?;
    let (after_quotes, name) = read_inside_quotes(name_start)?;

    assert_eq!(name, expected_data);

    let (format_start, _) = take_until_consume(after_quotes, b"format=")?;
    let (mut after_format, format_name) = read_inside_quotes(format_start)?;

    let header = match format_name {
        b"appended" => {
            // we also need the offset header so we know when to start reading
            let (offset_start, _) = take_until_consume(after_format, b"offset=")?;
            let (rest, offset) = read_inside_quotes(offset_start)?;
            after_format = rest;

            // TODO: better error handling for this
            let offset_str = std::str::from_utf8(offset).unwrap();
            let offset = offset_str.parse().expect(&format!(
                "data array offset `{}` coult not be parsed as integer",
                offset_str
            ));
            DataArrayHeader::AppendedBinary { offset }
        }
        b"binary" => {
            // we have base64 encoded data here
            DataArrayHeader::InlineBase64
        }
        b"ascii" => {
            // plain ascii data here
            DataArrayHeader::InlineAscii
        }
        _ => {
            // TODO: find a better way to make errors here
            let (_, _): (&[u8], &[u8]) = tag(
                "missing formatting header as appended/binary/ascii".as_bytes()
            )("".as_bytes())?;
            unreachable!()
        }
    };

    let (rest, _) = take_until_consume(after_format, b">")?;

    Ok((rest, header))
}

fn take_until_consume<'a>(input: &'a [u8], until_str: &[u8]) -> IResult<&'a [u8], ()> {
    let (non_consumed, _other) = take_until(until_str)(input)?;
    let (consumed, _format_header) = tag(until_str)(non_consumed)?;
    Ok((consumed, ()))
}

/// reads the data inside of two `"` characters, consuming the quotes in the process
fn read_inside_quotes<'a>(i: &'a [u8]) -> IResult<&'a [u8], &'a [u8]> {
    let (after_quote, _quote_char) = tag("\"")(i)?;
    let (after_inner, inner_data) = take_till(|c| c == b'"')(after_quote)?;
    let (after_quote, _quote_char) = tag("\"")(after_inner)?;
    Ok((after_quote, inner_data))
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
/// Describes what kind of information is in a header
pub enum DataArrayHeader {
    /// Ascii information is contained directly within the `DataArray` elements
    InlineAscii,
    /// Base64 information is contained directly within the `DataArray` elements
    InlineBase64,
    /// Information is not stored inline, it is stored at a specified `offset` 
    /// in the `AppendedData` section
    AppendedBinary { offset: i64 },
}

#[derive(Debug)]
/// Describes if the data for this array has already been parsed (regardless of format), or its offset
/// in the `AppendedData` section
pub enum PartialDataArray {
    Parsed(Vec<f64>),
    AppendedBinary { offset: i64 },
}

impl PartialDataArray {
    /// unwrap the data as `PartailDataArray::Parsed` or panic
    pub fn unwrap_parsed(self) -> Vec<f64> {
        match self {
            Self::Parsed(x) => x,
            _ => panic!("called unwrap_parsed on a PartialDataArray::AppendedBinary"),
        }
    }

    /// unwrap the data as `PartailDataArray::AppendedBinary` or panic
    pub fn unwrap_appended(self) -> i64 {
        match self {
            Self::AppendedBinary { offset } => offset,
            _ => panic!("called unwrap_parsed on a PartialDataArray::AppendedBinary"),
        }
    }
}

/// Similar to `PartialDataArray`, but instead contains an allocation for 
/// data to be placed for the `AppendedBinary` section. 
///
/// Useful for implementing `traits::ParseDataArray`
pub enum PartialDataArrayBuffered {
    Parsed(Vec<f64>),
    AppendedBinary(OffsetBuffer),
}

impl<'a> PartialDataArrayBuffered {
    /// Construct a buffer associated with appended binary 
    pub fn new(partial: PartialDataArray, size_hint: usize) -> Self {
        match partial {
            PartialDataArray::Parsed(x) => PartialDataArrayBuffered::Parsed(x),
            PartialDataArray::AppendedBinary { offset } => {
                PartialDataArrayBuffered::AppendedBinary(OffsetBuffer {
                    offset,
                    buffer: Vec::with_capacity(size_hint),
                })
            }
        }
    }

    /// Pull the data buffer from from each 
    /// of the variants
    pub fn into_buffer(self) -> Vec<f64> {
        match self {
            Self::Parsed(x) => x,
            Self::AppendedBinary(offset_buffer) => offset_buffer.buffer,
        }
    }
}

#[derive(PartialEq, PartialOrd)]
/// Helper struct describing the offset that the data should be read at
/// and the buffer that will be used to read in the information
pub struct OffsetBuffer {
    pub offset: i64,
    pub buffer: Vec<f64>,
}

impl Eq for OffsetBuffer {}

impl Ord for OffsetBuffer {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.offset.cmp(&other.offset)
    }
}

/// parse the values for a single inline ascii encoded array
///
/// ensure that before calling this function you have verified
/// that the data is base64 encoded with a call to `read_dataarray_header`
fn parse_ascii_inner_dataarray<'a>(
    xml_bytes: &'a [u8],
    size_hint: usize,
) -> IResult<&'a [u8], Vec<f64>> {

    let (location_data_and_rest, _whitespace) =
        take_till(|c: u8| c.is_ascii_digit() || c == b'.' || c == b'-')(xml_bytes)?;
    let (rest_of_document, location_data) = take_till(|c| c == b'<')(location_data_and_rest)?;

    let location_data_string =
        std::str::from_utf8(location_data).expect("ascii data was not encoded as UTF-8");

    let mut out = Vec::with_capacity(size_hint);

    location_data_string
        .trim_end()
        .split_ascii_whitespace()
        .for_each(|x| {
            let num = x
                .parse()
                .expect(&format!("ascii number {} could not be parsed as such", x));
            out.push(num);
        });

    Ok((rest_of_document, out))
}

/// parse the values for a single inline base64 encoded array
///
/// ensure that before calling this function you have verified
/// that the data is base64 encoded with a call to `read_dataarray_header`
fn parse_base64_inner_dataarray<'a>(
    xml_bytes: &'a [u8],
    size_hint: usize,
) -> IResult<&'a [u8], Vec<f64>> {
    let (rest_of_document, base64_encoded_bytes) = take_until("</D")(xml_bytes)?;
    let mut out = Vec::with_capacity(size_hint);

    let numerical_bytes =
        base64::decode(&base64_encoded_bytes).expect("could not decode base64 data array bytes");


    // normally we start with idx = 0, but since paraview expects the first 8 bytes 
    // to be garbage information we need to skip the first 8 bytes before actually
    // reading the data
    let mut idx = 8;
    let inc = 8;

    loop {
        if let Some(byte_slice) = numerical_bytes.get(idx..idx + inc) {
            if byte_slice.len() != 8 {
                break;
            }

            let mut const_slice = [0; 8];
            // copy in the slice to a fixed size array
            // could use unsafe here if we really wanted to
            byte_slice
                .iter()
                .enumerate()
                .for_each(|(slice_index, value)| const_slice[slice_index] = *value);

            let float = f64::from_le_bytes(const_slice);
            out.push(float);
        } else {
            break;
        }

        idx += inc;
    }

    Ok((rest_of_document, out))
}

/// skip to the appended data section so that we can read in the binary
///
/// Call this function after reading all of the information from the inline data arrays
pub fn setup_appended_read<'a>(xml_bytes: &[u8]) -> IResult<&[u8], ()> {
    // TODO: make this function return the type of encoding used in the appended section
    let (appended_data_section, _) = take_until_consume(xml_bytes, b"AppendedData")?;
    let (appended_start, _encoding_information) = take_until_consume(appended_data_section, b">_")?;
    let (appended_start_no_garbage, _) = take(8usize)(appended_start)?;
    Ok((appended_start_no_garbage, ()))
}

pub enum AppendedArrayLength {
    Known(usize),
    UntilEnd,
}

/// read information from the appended data binary buffer
pub fn parse_appended_binary<'a>(
    xml_bytes: &'a [u8],
    length: AppendedArrayLength,
    parsed_bytes: &mut Vec<f64>,
) -> IResult<&'a [u8], ()> {
    let (rest, bytes) = match length {
        AppendedArrayLength::Known(known_length) => {
            let (rest_of_appended, current_bytes_slice) = take(known_length)(xml_bytes)?;
            (rest_of_appended, current_bytes_slice)
        }
        AppendedArrayLength::UntilEnd => {
            let (rest_of_appended, current_bytes_slice) =
                take_until(b"</Appended".as_ref())(xml_bytes)?;
            (rest_of_appended, current_bytes_slice)
        }
    };

    let mut idx = 0;
    let inc = 8;

    loop {
        if let Some(byte_slice) = bytes.get(idx..idx + inc) {
            let float = utils::bytes_to_float(byte_slice);
            parsed_bytes.push(float);
        } else {
            break;
        }

        idx += inc;
    }

    Ok((rest, ()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shred_to_extent() {
        let input = r#"<VTKFile type="RectilinearGrid" version="1.0" byte_order="LittleEndian" header_type="UInt64">
            <RectilinearGrid WholeExtent="1 220 1 200 1 1">
            <Piece Extent="1 220 1 200 1 1">"#;
        let out = find_extent(input.as_bytes());
        out.unwrap();
    }

    #[test]
    fn shred_to_locations() {
        let locations = LocationSpans {
            x_start: 1,
            x_end: 4,
            y_start: 1,
            y_end: 4,
            z_start: 1,
            z_end: 4,
        };

        let input = r#"
            <Piece Extent="1 220 1 200 1 1">
            <Coordinates>
                <DataArray type="Float64" NumberOfComponents="1" Name="X" format="ascii">
                    .0000000000E+00 .3981797497E-01 .7963594994E-01 .1194539249E+00
                </DataArray>
                <DataArray type="Float64" NumberOfComponents="1" Name="Y" format="ascii">
                    .0000000000E+00 .3981797497E-01 .7963594994E-01 .1194539249E+00
                </DataArray>
                <DataArray type="Float64" NumberOfComponents="1" Name="Z" format="ascii">
                    .0000000000E+00 .3981797497E-01 .7963594994E-01 .1194539249E+00
                </DataArray>
            "#;
        let out = parse_locations(input.as_bytes(), &locations);
        let out = out.unwrap().1;

        assert_eq!(out.x.unwrap_parsed().len(), 4);
        assert_eq!(out.y.unwrap_parsed().len(), 4);
        assert_eq!(out.z.unwrap_parsed().len(), 4);
    }

    #[test]
    fn single_location() {
        let input = r#"
                <DataArray type="Float64" NumberOfComponents="1" Name="X" format="ascii">
                    .0000000000E+00 .3981797497E-01 .7963594994E-01 .1194539249E+00
                </DataArray>
            "#;
        let out = parse_dataarray_or_lazy(input.as_bytes(), b"X", 4);
        dbg!(&out);
        let out = out.unwrap();
        let expected = 4;

        assert_eq!(out.1.unwrap_parsed().len(), expected);
    }

    #[test]
    fn full_vtk_ascii() {
        let out = read_and_parse(std::path::Path::new("./static/sample_vtk_file.vtk"));
        dbg!(&out);
        let out: crate::VtkData<crate::helpers::SpanData> = out.unwrap();
        assert!(out.data.u.len() > 1000);
    }

    #[test]
    fn full_vtk_base64() {
        let out = read_and_parse(std::path::Path::new("./static/base64.vtk"));
        dbg!(&out);
        let out: crate::VtkData<crate::helpers::SpanData> = out.unwrap();
        assert!(out.data.u.len() > 1000);
    }

    #[test]
    fn full_vtk_binary() {
        let out = read_and_parse(std::path::Path::new("./static/binary.vtk"));
        dbg!(&out);
        let out: crate::VtkData<crate::helpers::SpanDataBinary> = out.unwrap();
        assert!(out.data.u.len() > 1000);
        assert!(out.data.v.len() > 1000);
        assert!(out.data.w.len() > 1000);
    }

    #[test]
    fn check_inside_quote() {
        let data = r#""quote_data""#;
        let out = read_inside_quotes(data.as_bytes());
        dbg!(&out);
        let (_, out) = out.unwrap();
        assert_eq!(out, b"quote_data");
    }

    #[test]
    fn ascii_array_header() {
        let header = r#"<DataArray type="Float64" NumberOfComponents="1" Name="X" format="ascii">"#;
        let out = read_dataarray_header(header.as_bytes(), b"X");
        dbg!(&out);

        let (rest, array_type) = out.unwrap();

        assert_eq!(array_type, DataArrayHeader::InlineAscii);
        assert_eq!(rest, b"");
    }

    #[test]
    fn base64_array_header() {
        let header =
            r#"<DataArray type="Float64" NumberOfComponents="1" Name="X" format="binary">"#;
        let out = read_dataarray_header(header.as_bytes(), b"X");
        dbg!(&out);

        let (rest, array_type) = out.unwrap();

        assert_eq!(array_type, DataArrayHeader::InlineBase64);
        assert_eq!(rest, b"");
    }

    #[test]
    fn appended_array_header() {
        let header = r#"<DataArray type="Float64" NumberOfComponents="1" Name="X" format="appended" offset="99">"#;
        let out = read_dataarray_header(header.as_bytes(), b"X");
        dbg!(&out);

        let (rest, array_type) = out.unwrap();

        assert_eq!(array_type, DataArrayHeader::AppendedBinary { offset: 99 });
        assert_eq!(rest, b"");
    }

    #[test]
    fn base_64_encoded_array() {
        let values = [1.0, 2.0, 3.0, 4.0];
        let mut output = Vec::new();
        let mut event_writer = crate::EventWriter::new(&mut output);
        crate::write_inline_dataarray(&mut event_writer, &values, "X", crate::Encoding::Base64)
            .unwrap();

        let string = String::from_utf8(output).unwrap();
        let parsed_result = parse_dataarray_or_lazy(&string.as_bytes(), b"X", 4);

        dbg!(&parsed_result);

        let out = parsed_result.unwrap();

        assert_eq!(out.1.unwrap_parsed(), &values);
    }

    #[test]
    fn appended_array() {
        let values = [1.0, 2.0, 3.0, 4.0];
        let values2 = [5.0, 6.0, 7.0, 8.0];

        let mut output = Vec::new();
        let mut event_writer = crate::EventWriter::new(&mut output);

        let offset_1 = -8;
        let offset_2 = -8 + (4 * 8);

        crate::write_appended_dataarray_header(&mut event_writer, "X", offset_1).unwrap();
        crate::write_appended_dataarray_header(&mut event_writer, "Y", offset_2).unwrap();

        // write the data inside the appended section
        crate::write_vtk::appended_binary_header_start(&mut event_writer).unwrap();

        // need to write a single garbage byte for things to work as expected - this 
        // is becasue of how paraview expects things
        crate::write_appended_dataarray(&mut event_writer, &[100f64]).unwrap();

        crate::write_appended_dataarray(&mut event_writer, &values).unwrap();
        crate::write_appended_dataarray(&mut event_writer, &values2).unwrap();

        crate::write_vtk::appended_binary_header_end(&mut event_writer).unwrap();

        // now we can start parsing the data
        let string_representation = String::from_utf8_lossy(&output);
        println!(":: xml data - {} ", string_representation);

        // write data array headers
        let (rest, parsed_header_1) = parse_dataarray_or_lazy(&output, b"X", 4).unwrap();
        let (rest, parsed_header_2) = parse_dataarray_or_lazy(&rest, b"Y", 4).unwrap();

        let header_1 = parsed_header_1.unwrap_appended();
        let header_2 = parsed_header_2.unwrap_appended();

        let len_1 = AppendedArrayLength::Known((header_2 - header_1) as usize);
        let len_2 = AppendedArrayLength::UntilEnd;

        let (rest, _) = setup_appended_read(rest).unwrap();

        println!(
            ":: xml data after queued movement- {} ",
            string_representation
        );

        let mut data_1 = Vec::new();
        let mut data_2 = Vec::new();

        let (rest, _) = parse_appended_binary(rest, len_1, &mut data_1).unwrap();

        let string_representation = String::from_utf8_lossy(&rest);
        println!("between parses - {}", string_representation);
        let (_rest, _) = parse_appended_binary(rest, len_2, &mut data_2).unwrap();

        assert_eq!(values.as_ref(), data_1);
        assert_eq!(values2.as_ref(), data_2);
    }
}
