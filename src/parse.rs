//! reading and parsing xml VTK files
//!
//! most of the time you will not need to interact with this file,
//! instead derive `ParseDataArray`

use crate::prelude::*;
use crate::utils;
use nom::bytes::complete::{tag, take, take_till, take_until};

use std::fmt;
use std::io::Read;
use std::io::BufRead;
use std::io::BufReader;
use std::fs::File;

use quick_xml::reader::Reader;
use quick_xml::events::attributes::Attribute;
use quick_xml::events::BytesEnd;
use quick_xml::events::BytesStart;
use quick_xml::events::Event;
use quick_xml::name::QName;

type NomErr<'a> = nom::Err<nom::error::Error<&'a [u8]>>;

/// An error caused from parsing the vtk files
#[derive(Debug, thiserror::Error)]
pub struct ParseError {
    nom_reason: Vec<u8>,
    nom_code: nom::error::ErrorKind,
    extra_info: &'static str,
}

#[derive(Debug, thiserror::Error, From)]
pub enum NeoParseError {
    #[error("Error parsing vtk file before coordinate section: {0}")]
    Header(Header),
    #[error("Error parsing vtk file before coordinate section: {0}")]
    RectilinearHeader(RectilinearHeader),
    #[error("Error parsing vtk file before coordinate section: {0}")]
    CoordinatesHeader(CoordinatesHeader),
    #[error("Error parsing vtk file before coordinate section: {0}")]
    Mesh(Mesh),
}

#[derive(Debug, thiserror::Error, From)]
pub enum Header {
    #[error("{0}")]
    MalformedXml(MalformedXml),
    #[error("{0}")]
    MalformedAttribute(MalformedAttribute),
    #[error("{0}")]
    UnexpectedElement(UnexpectedElement),
    #[error("{0}")]
    UnexpectedAttributeValue(UnexpectedAttributeValue),
}

#[derive(From, Display, Debug)]
#[display(fmt = "failed to parse an xml element: {xml_err}")]
pub struct MalformedXml {
    xml_err: quick_xml::Error
}

#[derive(From, Display, Debug)]
#[display(fmt = "failed to parse an xml attribute: {att_err}")]
pub struct MalformedAttribute {
    att_err: quick_xml::events::attributes::AttrError
}

#[derive(From, Display, Debug, Constructor)]
#[display(fmt = "unexpected element name `{actual_element}` occured in VTK file before `{expected_name}`")]
pub struct UnexpectedElement {
    expected_name: String,
    actual_element: ParsedNameOrBytes,
}

#[derive(From, Display, Debug, Constructor)]
#[display(fmt = "unexpected attribute value for {attribute_name} in {element_name} element: expected {expected_value}, got {actual_value}")]
pub struct UnexpectedAttributeValue {
    element_name: String,
    attribute_name: String,
    expected_value: String,
    actual_value: ParsedNameOrBytes,
}

#[derive(From, Display, Debug, Constructor)]
#[display(fmt = "missing attribute `{attribute_name}` in {element_name} element")]
pub struct MissingAttribute {
    element_name: String,
    attribute_name: String,
}

#[derive(From, Display, Debug)]
pub enum ParsedNameOrBytes {
    #[display(fmt = "{_0}")]
    Utf8(String),
    #[display(fmt = "{_0:?} (cannot convert to UTF8 string)")]
    Bytes(Vec<u8>)
}

impl ParsedNameOrBytes {
    fn new(bytes: &[u8]) -> Self {
        let vec = Vec::from(bytes);
        match String::from_utf8(vec) {
            Ok(string) => Self::Utf8(string),
            Err(e) => Self::Bytes(e.into_bytes())
        } 
    }
}

impl <'a> From<QName<'a>> for ParsedNameOrBytes {
    fn from(x: QName) -> Self {
        Self::new(x.as_ref())
    }
}

impl <'a> From<std::borrow::Cow<'a, [u8]>> for ParsedNameOrBytes {
    fn from(x: std::borrow::Cow<'a, [u8]>) -> Self {
        Self::new(x.as_ref())
    }
}

impl <'a> From<&'a str> for ParsedNameOrBytes {
    fn from(x: &str) -> Self {
        Self::Utf8(x.into())
    }
}

#[derive(Debug, thiserror::Error, From)]
pub enum RectilinearHeader {
    #[error("{0}")]
    MalformedXml(MalformedXml),
    #[error("{0}")]
    MalformedAttribute(MalformedAttribute),
    #[error("{0}")]
    MissingAttribute(MissingAttribute),
    #[error("{0}")]
    UnexpectedElement(UnexpectedElement),
    #[error("{0}")]
    UnexpectedAttributeValue(UnexpectedAttributeValue),
}

#[derive(Debug, thiserror::Error, From)]
pub enum CoordinatesHeader {
    #[error("{0}")]
    MalformedXml(MalformedXml),
    #[error("{0}")]
    MalformedAttribute(MalformedAttribute),
    #[error("{0}")]
    MissingAttribute(MissingAttribute),
    #[error("{0}")]
    UnexpectedElement(UnexpectedElement),
    #[error("{0}")]
    UnexpectedAttributeValue(UnexpectedAttributeValue),
}

#[derive(Debug, thiserror::Error, From)]
pub enum Mesh {
    #[error("{0}")]
    MalformedXml(MalformedXml),
    #[error("{0}")]
    MalformedAttribute(MalformedAttribute),
    #[error("{0}")]
    MissingAttribute(MissingAttribute),
    #[error("{0}")]
    UnexpectedElement(UnexpectedElement),
    #[error("{0}")]
    UnexpectedAttributeValue(UnexpectedAttributeValue),
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
pub fn read_and_parse<GEOMETRY, SPAN, D, MESH, ArrayVisitor, MeshVisitor>(
    path: &std::path::Path,
) -> Result<VtkData<GEOMETRY, D>, Error>
where
    D: ParseArray<Visitor = ArrayVisitor>,
    ArrayVisitor: Visitor<SPAN, Output = D>,
    MESH: ParseMesh<Visitor = MeshVisitor>,
    MeshVisitor: Visitor<SPAN, Output = MESH>,
    SPAN: ParseSpan,
    GEOMETRY: From<(MESH, SPAN)>,
{
    let file = std::fs::File::open(path)?;
    let buf_reader = std::io::BufReader::new(file);
    let reader = Reader::from_reader(buf_reader);

    parse_xml_document(reader)
}

fn read_to_grid_header<R: BufRead>( reader: &mut Reader<R>, buffer: &mut Vec<u8>) -> Result<(), Header> { 
    // find a VTKFile leading element
    loop {
        let event = reader.read_event_into(buffer)
            .map_err(MalformedXml::from)?;

        if let Event::Start(inner_start) = &event {
            if inner_start.name() != QName(b"VTKFile")  {
                let element_mismatch = UnexpectedElement::new("VTKFile".into(), ParsedNameOrBytes::from(inner_start.name()));
                return Err(Header::from(element_mismatch))
            }

            // now we know the element has the correct name, now we check that its RectilinearGrid
            // the correct version, and the correct byte encoding

            let attributes = inner_start.attributes();

            for attribute in attributes {
                let attribute = attribute.map_err(MalformedAttribute::from)?;

                // check the type of the element
                if attribute.key.as_ref() == b"type" {
                    check_attribute_value(attribute, "VTKFile", "type", "RectilinearGrid")?;
                }
                else if attribute.key.as_ref() == b"byte_order" {
                    check_attribute_value(attribute, "VTKFile", "byte_order", "LittleEndian")?;
                }
            }
        }

        // catch an EOF if we are looping
        if let Event::Eof = event {
            let element_mismatch = UnexpectedElement::new("VTKFile".into(), ParsedNameOrBytes::Utf8("EOF".into()));
            return Err(Header::from(element_mismatch))
        }
    }
}

/// parse the RectilinearGrid element header, return the contents of the `WholeExtent` attribute
fn read_rectilinear_header<SPAN: ParseSpan, R: BufRead>(reader: &mut Reader<R>, buffer: &mut Vec<u8>) -> Result<SPAN, RectilinearHeader> {
    let event = read_starting_element_with_name::<RectilinearHeader, _>(reader, buffer, "RectilinearGrid")?;

    let extent_value = get_attribute_value::<RectilinearHeader>(event, "WholeExtent", "RectilinearGrid")?;
    let extent_str = String::from_utf8(extent_value).unwrap();
    Ok(SPAN::from_str(&extent_str))
}

fn read_to_coordinates<SPAN: ParseSpan, R: BufRead>(reader: &mut Reader<R>, buffer: &mut Vec<u8>) -> Result<SPAN, CoordinatesHeader> {
    let piece = read_starting_element_with_name::<CoordinatesHeader, _>(reader, buffer, "Piece")?;

    let extent_value = get_attribute_value::<CoordinatesHeader>(piece, "Extent", "Piece")?;
    let extent_str = String::from_utf8(extent_value).unwrap();
    let extent = SPAN::from_str(&extent_str);

    // then, we read the next element which should be the `Coordinates` element, which
    // indicates that we are about to start reading the grid elements
    let _coordinates =read_starting_element_with_name::<CoordinatesHeader, _>(reader, buffer, "Coordinates")?;
    //reading the closing of this element will be handled later

    Ok(extent)
}

fn read_starting_element_with_name<'a, E, R: BufRead>(reader: &mut Reader<R>, buffer: &'a mut Vec<u8>, expected_name: &str) -> Result<BytesStart<'a>, E> 
where E: From<UnexpectedElement> + From<MalformedXml>
{
    let element = reader.read_event_into(buffer)
        .map_err(MalformedXml::from)?;

    let event = if let Event::Start(event) = element {
        event
    } else{
        let unexpected = UnexpectedElement::new(expected_name.into(), ParsedNameOrBytes::from("non starting element"));
        return Err(E::from(unexpected))
    };

    // check that the name of the coordinates header is correct
    if event.name().as_ref() != expected_name.as_bytes() {
        let unexpected = UnexpectedElement::new(expected_name.into(), ParsedNameOrBytes::from(event.name()));
        return Err(E::from(unexpected))
    }

    Ok(event)
}

fn get_attribute_value<'a, E>(bytes_start: BytesStart<'a>, attribute_key: &str, element_name: &str) -> Result<Vec<u8>, E> 
where E: From<MissingAttribute>{
    // find the `attribute_key` attribute on the `element_name` element
    let extent = bytes_start.attributes()
        // TODO: error here if there was a malformed attribute
        .filter_map(|x| x.ok())
        .find(|x| x.key.as_ref() == attribute_key.as_bytes());

    if let Some(extent) = extent {
        Ok(extent.value.to_vec())
    } else {
        let err=  MissingAttribute::new(element_name.into(), attribute_key.into());
        Err(E::from(err))
    }
}

/// ensure that an attribute's value is what we expect it to be, otherwise return an error with
/// some location information
fn check_attribute_value<'a>(att: Attribute<'a>, element_name: &str, attribute_name: &str, expected_attribute_value: &str) -> Result<(), UnexpectedAttributeValue> {
    if att.value.as_ref() != expected_attribute_value.as_bytes() {
        let unexpected_value = UnexpectedAttributeValue {
            element_name: element_name.into(),
            attribute_name: attribute_name.into(),
            expected_value: expected_attribute_value.into(),
            actual_value: ParsedNameOrBytes::from(att.value),
        };

        Err(unexpected_value)
    } else {
        Ok(())
    }
}

#[doc(hidden)]
pub fn parse_xml_document<DOMAIN, SPAN, D, MESH, ArrayVisitor, MeshVisitor, R: BufRead>(
    mut reader: Reader<R>
) -> Result<VtkData<DOMAIN, D>, Error>
where
    D: ParseArray<Visitor = ArrayVisitor>,
    ArrayVisitor: Visitor<SPAN, Output = D>,
    MESH: ParseMesh<Visitor = MeshVisitor>,
    MeshVisitor: Visitor<SPAN, Output = MESH>,
    SPAN: ParseSpan,
    DOMAIN: From<(MESH, SPAN)>,
{
    let mut buffer = Vec::new();

    let _ = read_to_grid_header(&mut reader, &mut buffer)
        .map_err(NeoParseError::from)?;

    // find the whole extent from the RectilinearGrid header
    let spans = read_rectilinear_header::<SPAN, _>(&mut reader, &mut buffer)
        .map_err(NeoParseError::from)?;

    let local_spans = read_to_coordinates::<SPAN, _>(&mut reader, &mut buffer)
        .map_err(NeoParseError::from)?;

    let location_visitor =
        MeshVisitor::read_headers(&spans, &mut reader, &mut buffer)
            .map_err(NeoParseError::from)?;

    let array_visitor = ArrayVisitor::read_headers(&spans, &mut reader, &mut buffer)
        .map_err(NeoParseError::from)?;

    todo!()

    //let mut reader_buffer = Vec::new();
    //location_visitor.add_to_appended_reader(&mut reader_buffer);
    //array_visitor.add_to_appended_reader(&mut reader_buffer);

    //read_appended_array_buffers(reader_buffer, rest)?;

    //let data: D = array_visitor.finish(&spans)?;
    //let mesh: MESH = location_visitor.finish(&spans)?;
    //let domain = DOMAIN::from((mesh, spans));

    //Ok(VtkData { domain, data })
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

pub(crate) fn find_extent<SPAN: ParseSpan>(i: &[u8]) -> IResult<&[u8], SPAN> {
    let (start_extent, _xml_header_info) = take_until("WholeExtent")(i)?;
    let (extent_string_start, _whole_extent_header) = tag("WholeExtent=\"")(start_extent)?;
    let (rest_of_document, extent_string) = take_till(|c| c == b'\"')(extent_string_start)?;

    let spans = SPAN::from_str(std::str::from_utf8(extent_string).unwrap());

    Ok((rest_of_document, spans))
}

/// Parse a data array (if its inline) or return the offset in the appended section
pub fn parse_dataarray_or_lazy<'a, R: BufRead>(
    reader: &mut Reader<R>,
    buffer: &mut Vec<u8>,
    expected_name: &str,
    size_hint: usize,
) -> Result<PartialDataArray, Mesh> {
    todo!()
    //let (mut rest, header) = read_dataarray_header(xml_bytes, expected_data)?;
    //let lazy_array = match header {
    //    DataArrayHeader::AppendedBinary { offset, components } => {
    //        PartialDataArray::AppendedBinary { offset, components }
    //    }
    //    DataArrayHeader::InlineAscii { components } => {
    //        let (after_dataarray, parsed_data) = parse_ascii_inner_dataarray(rest, size_hint)?;
    //        rest = after_dataarray;
    //        PartialDataArray::Parsed {
    //            buffer: parsed_data,
    //            components,
    //        }
    //    }
    //    DataArrayHeader::InlineBase64 { components } => {
    //        let (after_dataarray, parsed_data) = parse_base64_inner_dataarray(rest, size_hint)?;
    //        rest = after_dataarray;
    //        PartialDataArray::Parsed {
    //            buffer: parsed_data,
    //            components,
    //        }
    //    }
    //};

    //Ok((rest, lazy_array))
}

/// cycle through buffers (and their offsets) and read the binary information from the
/// <AppendedBinary> section in order
pub fn read_appended_array_buffers(
    mut buffers: Vec<RefMut<'_, OffsetBuffer>>,
    bytes: &[u8],
) -> Result<(), ParseError> {
    // if we have any binary data:
    if buffers.len() > 0 {
        //we have some data to read - first organize all of the data by the offsets
        buffers.sort_unstable_by_key(|x| x.offset);

        let mut iterator = buffers.iter_mut().peekable();
        let (mut appended_data, _) = crate::parse::setup_appended_read(bytes)?;

        loop {
            if let Some(current_offset_buffer) = iterator.next() {
                // get the number of bytes to read based on the next element's offset
                let reading_offset = iterator
                    .peek()
                    .map(|offset_buffer| {
                        let diff = offset_buffer.offset - current_offset_buffer.offset;
                        crate::parse::AppendedArrayLength::Known((diff) as usize)
                    })
                    .unwrap_or(crate::parse::AppendedArrayLength::UntilEnd);

                let (remaining_appended_data, _) = crate::parse::parse_appended_binary(
                    appended_data,
                    reading_offset,
                    &mut current_offset_buffer.buffer,
                )?;

                appended_data = remaining_appended_data
            } else {
                // there are not more elements in the array - lets leave
                break;
            }
        }
    }

    Ok(())
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
pub fn read_dataarray_header<'a, R: BufRead>(
    reader: &mut Reader<R>,
    buffer: &mut Vec<u8>,
    expected_name: &str,
) -> Result<DataArrayHeader, Mesh> {

    let array_start = read_starting_element_with_name::<Mesh, _>(reader, buffer, expected_name)?;

    todo!()
    
    //let num_components = 

    //// grab the number of components as well
    //let (components_start, _) = take_until_consume(xml_bytes, b"NumberOfComponents=")?;
    //let (after_components, num_components_str) = read_inside_quotes(components_start)?;

    //let components = String::from_utf8(num_components_str.to_vec())
    //    .unwrap()
    //    .parse()
    //    .unwrap();

    //let (name_start, _) = take_until_consume(after_components, b"Name=")?;
    //let (after_quotes, name) = read_inside_quotes(name_start)?;

    //assert_eq!(name, expected_data);

    //let (format_start, _) = take_until_consume(after_quotes, b"format=")?;
    //let (mut after_format, format_name) = read_inside_quotes(format_start)?;

    //let header = match format_name {
    //    b"appended" => {
    //        // we also need the offset header so we know when to start reading
    //        let (offset_start, _) = take_until_consume(after_format, b"offset=")?;
    //        let (rest, offset) = read_inside_quotes(offset_start)?;
    //        after_format = rest;

    //        // TODO: better error handling for this
    //        let offset_str = std::str::from_utf8(offset).unwrap();
    //        let offset = offset_str.parse().expect(&format!(
    //            "data array offset `{}` coult not be parsed as integer",
    //            offset_str
    //        ));
    //        DataArrayHeader::AppendedBinary { offset, components }
    //    }
    //    b"binary" => {
    //        // we have base64 encoded data here
    //        DataArrayHeader::InlineBase64 { components }
    //    }
    //    b"ascii" => {
    //        // plain ascii data here
    //        DataArrayHeader::InlineAscii { components }
    //    }
    //    _ => {
    //        // TODO: find a better way to make errors here
    //        let (_, _): (&[u8], &[u8]) = tag(
    //            "missing formatting header as appended/binary/ascii".as_bytes()
    //        )("".as_bytes())?;
    //        unreachable!()
    //    }
    //};

    //let (rest, _) = take_until_consume(after_format, b">")?;

    //Ok((rest, header))
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
    InlineAscii { components: usize },
    /// Base64 information is contained directly within the `DataArray` elements
    InlineBase64 { components: usize },
    /// Information is not stored inline, it is stored at a specified `offset`
    /// in the `AppendedData` section
    AppendedBinary { offset: i64, components: usize },
}

#[derive(Debug)]
/// Describes if the data for this array has already been parsed (regardless of format), or its offset
/// in the `AppendedData` section
pub enum PartialDataArray {
    Parsed { buffer: Vec<f64>, components: usize },
    AppendedBinary { offset: i64, components: usize },
}

impl PartialDataArray {
    /// unwrap the data as `PartailDataArray::Parsed` or panic
    pub fn unwrap_parsed(self) -> Vec<f64> {
        match self {
            Self::Parsed { buffer, .. } => buffer,
            _ => panic!("called unwrap_parsed on a PartialDataArray::AppendedBinary"),
        }
    }

    /// unwrap the data as `PartailDataArray::AppendedBinary` or panic
    pub fn unwrap_appended(self) -> i64 {
        match self {
            Self::AppendedBinary { offset, .. } => offset,
            _ => panic!("called unwrap_parsed on a PartialDataArray::AppendedBinary"),
        }
    }

    /// unwrap the data as `PartailDataArray::AppendedBinary` or panic
    pub fn components(&self) -> usize {
        match self {
            Self::AppendedBinary { components, .. } => *components,
            Self::Parsed { components, .. } => *components,
        }
    }
}

/// Similar to `PartialDataArray`, but instead contains an allocation for
/// data to be placed for the `AppendedBinary` section.
///
/// Useful for implementing `traits::ParseDataArray`
pub enum PartialDataArrayBuffered {
    Parsed { buffer: Vec<f64>, components: usize },
    AppendedBinary(RefCell<OffsetBuffer>),
}

impl<'a> PartialDataArrayBuffered {
    /// Construct a buffer associated with appended binary
    pub fn new(partial: PartialDataArray, size_hint: usize) -> Self {
        match partial {
            PartialDataArray::Parsed { buffer, components } => {
                PartialDataArrayBuffered::Parsed { buffer, components }
            }
            PartialDataArray::AppendedBinary { offset, components } => {
                PartialDataArrayBuffered::AppendedBinary(RefCell::new(OffsetBuffer {
                    offset,
                    buffer: Vec::with_capacity(size_hint),
                    components,
                }))
            }
        }
    }

    /// Pull the data buffer from from each
    /// of the variants
    pub fn into_buffer(self) -> Vec<f64> {
        match self {
            Self::Parsed { buffer, .. } => buffer,
            Self::AppendedBinary(offset_buffer) => offset_buffer.into_inner().buffer,
        }
    }

    /// get the number of components associated with the header of the array
    pub fn components(&self) -> usize {
        match self {
            Self::Parsed { components, .. } => *components,
            Self::AppendedBinary(offset_buffer) => offset_buffer.borrow().components,
        }
    }

    /// helper function to put the array in a vector so that we can read all the binary data in
    /// order
    pub fn append_to_reader_list<'c, 'b>(&'c self, buffer: &'b mut Vec<RefMut<'c, OffsetBuffer>>) {
        match self {
            PartialDataArrayBuffered::AppendedBinary(offset_buffer) => {
                buffer.push(offset_buffer.borrow_mut())
            }
            // if this is here then we have already read the data inline, and we dont need to worry
            // about any appended data for this item
            _ => (),
        }
    }
}

#[derive(PartialEq, PartialOrd)]
/// Helper struct describing the offset that the data should be read at
/// and the buffer that will be used to read in the information
pub struct OffsetBuffer {
    pub offset: i64,
    pub buffer: Vec<f64>,
    pub components: usize,
}

impl Eq for OffsetBuffer {}

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
    use crate::Array;
    use crate::Binary;
    use crate::Mesh3D;
    use crate::Rectilinear3D;
    use crate::Spans3D;
    use crate::Visitor;
    type Domain = Rectilinear3D<f64, Binary>;

    #[test]
    fn shred_to_extent() {
        let input = r#"<VTKFile type="RectilinearGrid" version="1.0" byte_order="LittleEndian" header_type="UInt64">
            <RectilinearGrid WholeExtent="1 220 1 200 1 1">
            <Piece Extent="1 220 1 200 1 1">"#;
        let out = find_extent::<Spans3D>(input.as_bytes());
        out.unwrap();
    }

    #[test]
    fn shred_to_locations() {
        let spans = Spans3D::new(4, 4, 4);

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

        let (_rest, locations) =
            crate::mesh::Mesh3DVisitor::read_headers(&spans, input.as_bytes()).unwrap();
        let out = locations.finish(&spans).unwrap();

        assert_eq!(out.x_locations.len(), 4);
        assert_eq!(out.y_locations.len(), 4);
        assert_eq!(out.z_locations.len(), 4);
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
        let out: crate::VtkData<Domain, crate::helpers::SpanData> = out.unwrap();
        assert!(out.data.u.len() > 1000);
    }

    #[test]
    fn full_vtk_base64() {
        let out = read_and_parse(std::path::Path::new("./static/base64.vtk"));
        dbg!(&out);
        let out: crate::VtkData<Domain, crate::helpers::SpanData> = out.unwrap();
        assert!(out.data.u.len() > 1000);
    }

    #[test]
    #[cfg(feature = "derive")]
    fn full_vtk_binary() {
        use crate as vtk;
        #[derive(vtk::DataArray, vtk::ParseArray, Debug)]
        #[vtk_parse(spans = "vtk::Spans3D")]
        pub struct SpanDataBinary {
            u: Vec<f64>,
            v: Vec<f64>,
            w: Vec<f64>,
        }

        let out = read_and_parse(std::path::Path::new("./static/binary.vtk"));
        dbg!(&out);
        let out: crate::VtkData<Domain, SpanDataBinary> = out.unwrap();
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

        assert_eq!(array_type, DataArrayHeader::InlineAscii { components: 1 });
        assert_eq!(rest, b"");
    }

    #[test]
    fn base64_array_header() {
        let header =
            r#"<DataArray type="Float64" NumberOfComponents="1" Name="X" format="binary">"#;
        let out = read_dataarray_header(header.as_bytes(), b"X");
        dbg!(&out);

        let (rest, array_type) = out.unwrap();

        assert_eq!(array_type, DataArrayHeader::InlineBase64 { components: 1 });
        assert_eq!(rest, b"");
    }

    #[test]
    fn appended_array_header() {
        let header = r#"<DataArray type="Float64" NumberOfComponents="3" Name="X" format="appended" offset="99">"#;
        let out = read_dataarray_header(header.as_bytes(), b"X");
        dbg!(&out);

        let (rest, array_type) = out.unwrap();

        assert_eq!(
            array_type,
            DataArrayHeader::AppendedBinary {
                offset: 99,
                components: 3
            }
        );
        assert_eq!(rest, b"");
    }

    #[test]
    fn base_64_encoded_array() {
        let values = [1.0, 2.0, 3.0, 4.0];
        let mut output = Vec::new();
        let mut event_writer = crate::Writer::new(&mut output);
        crate::write_inline_dataarray(
            &mut event_writer,
            &values.as_slice(),
            "X",
            crate::Encoding::Base64,
        )
        .unwrap();

        let string = String::from_utf8(output).unwrap();
        let parsed_result = parse_dataarray_or_lazy(&string.as_bytes(), b"X", 4);

        dbg!(&parsed_result);

        let out = parsed_result.unwrap();

        assert_eq!(out.1.unwrap_parsed(), &values);
    }

    #[test]
    fn appended_array() {
        let values = [1.0f64, 2.0, 3.0, 4.0];
        let values2 = [5.0f64, 6.0, 7.0, 8.0];

        let mut output = Vec::new();
        let mut event_writer = crate::Writer::new(&mut output);

        let offset_1 = -8;
        let offset_2 = -8 + (4 * 8);

        crate::write_appended_dataarray_header(
            &mut event_writer,
            "X",
            offset_1,
            1,
            Precision::Float64,
        )
        .unwrap();
        crate::write_appended_dataarray_header(
            &mut event_writer,
            "Y",
            offset_2,
            1,
            Precision::Float64,
        )
        .unwrap();

        // write the data inside the appended section
        crate::write_vtk::appended_binary_header_start(&mut event_writer).unwrap();

        // need to write a single garbage byte for things to work as expected - this
        // is becasue of how paraview expects things
        [100f64]
            .as_ref()
            .write_binary(&mut event_writer, false)
            .unwrap();

        values
            .as_ref()
            .write_binary(&mut event_writer, false)
            .unwrap();
        values2
            .as_ref()
            .write_binary(&mut event_writer, true)
            .unwrap();

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
