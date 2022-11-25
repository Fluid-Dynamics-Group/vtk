//! reading and parsing xml VTK files
//!
//! most of the time you will not need to interact with this file,
//! instead derive `ParseDataArray`

mod error;
mod event_summary;

pub use error::Mesh;
pub use error::ParseError;
use event_summary::ElementName;
use event_summary::EventSummary;

use crate::prelude::*;
use crate::utils;
//use nom::bytes::complete::{tag, take, take_till, take_until};

use std::io::BufRead;

use quick_xml::events::attributes::Attribute;
use quick_xml::events::BytesEnd;
use quick_xml::events::BytesStart;
use quick_xml::events::BytesText;
use quick_xml::events::Event;
use quick_xml::name::QName;
use quick_xml::reader::Reader;

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

fn read_to_grid_header<R: BufRead>(
    reader: &mut Reader<R>,
    buffer: &mut Vec<u8>,
) -> Result<(), error::Header> {
    // find a VTKFile leading element
    loop {
        let event = reader
            .read_event_into(buffer)
            .map_err(error::MalformedXml::from)?;

        dbg!(&event);

        if let Event::Start(inner_start) = &event {
            if inner_start.name() != QName(b"VTKFile") {
                let actual_event = EventSummary::new(&event);

                let element_mismatch = error::UnexpectedElement::new("VTKFile", actual_event);
                return Err(error::Header::from(element_mismatch));
            }

            // now we know the element has the correct name, now we check that its RectilinearGrid
            // the correct version, and the correct byte encoding

            let attributes = inner_start.attributes();

            for attribute in attributes {
                let attribute = attribute.map_err(error::MalformedAttribute::from)?;

                // check the type of the element
                if attribute.key.as_ref() == b"type" {
                    check_attribute_value(attribute, "VTKFile", "type", "RectilinearGrid")?;
                } else if attribute.key.as_ref() == b"byte_order" {
                    check_attribute_value(attribute, "VTKFile", "byte_order", "LittleEndian")?;
                }
            }
        }

        // catch an EOF if we are looping
        if let Event::Eof = event {
            let actual_event = EventSummary::eof();

            let element_mismatch = error::UnexpectedElement::new("VTKFile", actual_event);

            return Err(error::Header::from(element_mismatch));
        }

        // sometimes there are headers for type of file, we just continue
        if let Event::Decl(_) = event {
            continue;
        }

        break;
    }

    Ok(())
}

/// parse the RectilinearGrid element header, return the contents of the `WholeExtent` attribute
fn read_rectilinear_header<SPAN: ParseSpan, R: BufRead>(
    reader: &mut Reader<R>,
    buffer: &mut Vec<u8>,
) -> Result<SPAN, error::RectilinearHeader> {
    let event = read_starting_element_with_name::<error::RectilinearHeader, _>(
        reader,
        buffer,
        "RectilinearGrid",
    )?;

    let extent_value =
        get_attribute_value::<error::RectilinearHeader>(&event, "WholeExtent", "RectilinearGrid")?;
    let extent_str = String::from_utf8(extent_value.value.to_vec()).unwrap();
    Ok(SPAN::from_str(&extent_str))
}

fn read_to_coordinates<SPAN: ParseSpan, R: BufRead>(
    reader: &mut Reader<R>,
    buffer: &mut Vec<u8>,
) -> Result<SPAN, error::CoordinatesHeader> {
    let piece =
        read_starting_element_with_name::<error::CoordinatesHeader, _>(reader, buffer, "Piece")?;

    let extent_value = get_attribute_value::<error::CoordinatesHeader>(&piece, "Extent", "Piece")?;
    let extent_str = String::from_utf8(extent_value.value.to_vec()).unwrap();
    let extent = SPAN::from_str(&extent_str);

    // then, we read the next element which should be the `Coordinates` element, which
    // indicates that we are about to start reading the grid elements
    let _coordinates = read_starting_element_with_name::<error::CoordinatesHeader, _>(
        reader,
        buffer,
        "Coordinates",
    )?;

    Ok(extent)
}

fn prepare_reading_point_data<R: BufRead>(
    reader: &mut Reader<R>,
    buffer: &mut Vec<u8>,
) -> Result<(), error::PreparePointData> {
    println!("closing </Coordinates> element");
    // first, we should have a closing element for Coordinates
    let _ = read_ending_element::<error::PreparePointData, _>(reader, buffer, "Coordinates")?;

    println!("finished closing </Coordinates> element, now opening PointData element");
    // then, we need to open the element for PointData
    let _ =
        read_starting_element_with_name::<error::PreparePointData, _>(reader, buffer, "PointData")?;
    println!("finished opening <PointData>");

    Ok(())
}

fn close_element_to_appended_data<R: BufRead>(
    reader: &mut Reader<R>,
    buffer: &mut Vec<u8>,
) -> Result<(), error::CloseElements> {
    println!("reading through to find </PointData> now");

    // first, we should have a closing element for PointData
    // however, we may have some unread <DataArray> elements that are still remaining
    // in the <PointData> section. These are allowed, so we just read through the VTK until
    // we find </PointData>
    let mut text_allowed = false;

    loop {
        println!("new element");
        let element = reader
            .read_event_into(buffer)
            .map_err(error::MalformedXml::from)?;

        let repr = EventSummary::new(&element);
        println!("element: {}", repr);

        match element {
            Event::Start(s) => {
                // we are opening a dataarray
                if s.byte_name().as_ref().unwrap().as_ref() == b"DataArray" {
                    text_allowed = true;
                    continue;
                }
            }
            Event::Empty(s) => {
                // we are opening a dataarray, but there is no inline data
                if s.byte_name().as_ref().unwrap().as_ref() == b"DataArray" {
                    // no text allowed, though
                    text_allowed = false;
                    continue;
                }
            }
            Event::End(s) => {
                // we have hit the point data, exit now
                if s.byte_name().as_ref().unwrap().as_ref() == b"PointData" {
                    break;
                }
                // we are closing a dataarray
                else if s.byte_name().as_ref().unwrap().as_ref() == b"DataArray" {
                    text_allowed = false;
                    continue;
                }
            }
            Event::Text(s) => {
                if text_allowed {
                    // we previously opened an element, now we can have text inside.
                    // this case may just be handled by the XML parser
                    continue;
                } else {
                    let actual = EventSummary::text(&s);
                    return Err(error::UnexpectedElement::new(
                        "PointData,DataArray,/DataArray",
                        actual,
                    )
                    .into());
                }
            }
            _ => {
                // we have hit something else,
                let actual = EventSummary::new(&element);
                return Err(error::UnexpectedElement::new(
                    "PointData,DataArray,/DataArray",
                    actual,
                )
                .into());
            }
        }
    }

    // then, we should have a </Piece>
    let _ = read_ending_element::<error::CloseElements, _>(reader, buffer, "Piece")?;

    // then, we should have a </RectilinearGrid>
    let _ = read_ending_element::<error::CloseElements, _>(reader, buffer, "RectilinearGrid")?;

    Ok(())
}

fn read_appended_data<R: BufRead>(
    reader: &mut Reader<R>,
    buffer: &mut Vec<u8>,
    reader_buffers: Vec<RefMut<'_, OffsetBuffer>>,
) -> Result<(), error::AppendedData> {
    // if there are no appended sections, we do not need to go on
    if reader_buffers.is_empty() {
        println!("skipping appended data section - no buffers to write to");
        return Ok(());
    }

    println!("starting read of appended data");
    let appended_data =
        read_starting_element_with_name::<error::AppendedData, _>(reader, buffer, "AppendedData")?;
    println!("finished queueing up to appended data section");

    let encoding =
        get_attribute_value::<error::AppendedData>(&appended_data, "encoding", "AppendedData")?;

    check_attribute_value(encoding, "AppendedData", "encoding", "raw")?;

    read_appended_array_buffers(reader, buffer, reader_buffers)?;

    Ok(())
}

fn read_starting_element_with_name<'a, E, R: BufRead>(
    reader: &mut Reader<R>,
    buffer: &'a mut Vec<u8>,
    expected_name: &str,
) -> Result<BytesStart<'a>, E>
where
    E: From<error::UnexpectedElement> + From<error::MalformedXml>,
{
    let element = reader
        .read_event_into(buffer)
        .map_err(error::MalformedXml::from)?;

    let event = if let Event::Start(event) = element {
        event
    } else {
        dbg!(&element);
        let actual_event = EventSummary::new(&element);

        let unexpected = error::UnexpectedElement::new(expected_name, actual_event);
        return Err(E::from(unexpected));
    };

    // check that the name of the coordinates header is correct
    if event.name().as_ref() != expected_name.as_bytes() {
        let actual_event = EventSummary::start(&event);
        let unexpected = error::UnexpectedElement::new(expected_name, actual_event);
        return Err(E::from(unexpected));
    }

    Ok(event)
}

fn read_empty_or_starting_element<'a, E, R: BufRead>(
    reader: &mut Reader<R>,
    buffer: &'a mut Vec<u8>,
    expected_name: &str,
) -> Result<(bool, BytesStart<'a>), E>
where
    E: From<error::UnexpectedElement> + From<error::MalformedXml>,
{
    let element = reader
        .read_event_into(buffer)
        .map_err(error::MalformedXml::from)?;

    let actual_event = EventSummary::new(&element);

    let (was_empty, event) = match element {
        Event::Empty(empty) => (true, empty),
        Event::Start(start) => (false, start),
        _ => {
            dbg!(&element);
            let actual_event = EventSummary::new(&element);
            let unexpected = error::UnexpectedElement::new(expected_name, actual_event);
            return Err(E::from(unexpected));
        }
    };

    // check that the name of the coordinates header is correct
    if event.name().as_ref() != expected_name.as_bytes() {
        let unexpected = error::UnexpectedElement::new(expected_name, actual_event);
        return Err(E::from(unexpected));
    }

    Ok((was_empty, event))
}

fn read_body_element<'a, E, R: BufRead>(
    reader: &mut Reader<R>,
    buffer: &'a mut Vec<u8>,
) -> Result<BytesText<'a>, E>
where
    E: From<error::UnexpectedElement> + From<error::MalformedXml>,
{
    let element = reader
        .read_event_into(buffer)
        .map_err(error::MalformedXml::from)?;

    let event = if let Event::Text(event) = element {
        event
    } else {
        let actual_event = EventSummary::new(&element);
        let unexpected = error::UnexpectedElement::new("body element", actual_event);
        return Err(E::from(unexpected));
    };

    Ok(event)
}

fn read_ending_element<'a, E, R: BufRead>(
    reader: &mut Reader<R>,
    buffer: &'a mut Vec<u8>,
    expected_name: &str,
) -> Result<BytesEnd<'a>, E>
where
    E: From<error::UnexpectedElement> + From<error::MalformedXml>,
{
    let element = reader
        .read_event_into(buffer)
        .map_err(error::MalformedXml::from)?;

    let event = if let Event::End(event) = element {
        event
    } else {
        dbg!(&element);
        let actual_event = EventSummary::new(&element);
        let unexpected = error::UnexpectedElement::new(format!("/{expected_name}"), actual_event);
        return Err(E::from(unexpected));
    };

    // check that the name of the coordinates header is correct
    if event.name().as_ref() != expected_name.as_bytes() {
        dbg!(&event);
        let actual_event = EventSummary::end(&event);
        let unexpected = error::UnexpectedElement::new(expected_name, actual_event);
        return Err(E::from(unexpected));
    }

    Ok(event)
}

fn get_attribute_value<'a, E>(
    bytes_start: &'a BytesStart<'_>,
    attribute_key: &str,
    element_name: &str,
) -> Result<Attribute<'a>, E>
where
    E: From<error::MissingAttribute>,
{
    // find the `attribute_key` attribute on the `element_name` element
    let extent = bytes_start
        .attributes()
        // TODO: error here if there was a malformed attribute
        .filter_map(|x| x.ok())
        .find(|x| x.key.as_ref() == attribute_key.as_bytes());

    if let Some(att) = extent {
        Ok(att)
    } else {
        let err = error::MissingAttribute::new(element_name.into(), attribute_key.into());
        Err(E::from(err))
    }
}

/// ensure that an attribute's value is what we expect it to be, otherwise return an error with
/// some location information
fn check_attribute_value<'a>(
    att: Attribute<'a>,
    element_name: &str,
    attribute_name: &str,
    expected_attribute_value: &str,
) -> Result<(), error::UnexpectedAttributeValue> {
    if att.value.as_ref() != expected_attribute_value.as_bytes() {
        let unexpected_value = error::UnexpectedAttributeValue {
            element_name: element_name.into(),
            attribute_name: attribute_name.into(),
            expected_value: expected_attribute_value.into(),
            actual_value: error::ParsedNameOrBytes::from(att.value),
        };

        Err(unexpected_value)
    } else {
        Ok(())
    }
}

#[doc(hidden)]
pub fn parse_xml_document<DOMAIN, SPAN, D, MESH, ArrayVisitor, MeshVisitor, R: BufRead>(
    mut reader: Reader<R>,
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

    // ignore whitespace in the reader
    reader.trim_text(true);

    let _ = read_to_grid_header(&mut reader, &mut buffer).map_err(ParseError::from)?;

    dbg!("finished reading to grid header");

    // find the whole extent from the RectilinearGrid header
    let spans =
        read_rectilinear_header::<SPAN, _>(&mut reader, &mut buffer).map_err(ParseError::from)?;

    let _local_spans =
        read_to_coordinates::<SPAN, _>(&mut reader, &mut buffer).map_err(ParseError::from)?;

    dbg!("reading locations");

    let location_visitor =
        MeshVisitor::read_headers(&spans, &mut reader, &mut buffer).map_err(ParseError::from)?;

    dbg!("finished reading locations");

    prepare_reading_point_data(&mut reader, &mut buffer).map_err(ParseError::from)?;

    println!("starting read for arrays: {}", line!());

    let array_visitor =
        ArrayVisitor::read_headers(&spans, &mut reader, &mut buffer).map_err(ParseError::from)?;

    println!("finished reading arrays: {}", line!());

    close_element_to_appended_data(&mut reader, &mut buffer).map_err(ParseError::from)?;

    let mut reader_buffer = Vec::new();
    location_visitor.add_to_appended_reader(&mut reader_buffer);
    array_visitor.add_to_appended_reader(&mut reader_buffer);

    read_appended_data(&mut reader, &mut buffer, reader_buffer).map_err(ParseError::from)?;

    let data: D = array_visitor.finish(&spans);
    let mesh: MESH = location_visitor.finish(&spans);
    let domain = DOMAIN::from((mesh, spans));

    Ok(VtkData { domain, data })
}

/// Parse a data array (if its inline) or return the offset in the appended section
pub fn parse_dataarray_or_lazy<'a, R: BufRead>(
    reader: &mut Reader<R>,
    buffer: &mut Vec<u8>,
    expected_name: &str,
    size_hint: usize,
) -> Result<PartialDataArray, Mesh> {
    println!("parse_dataarray_or_lazy, {}", line!());

    let (was_empty, header) = read_dataarray_header(reader, buffer, expected_name)?;

    let lazy_array = match header {
        DataArrayHeader::AppendedBinary { offset, components } => {
            PartialDataArray::AppendedBinary { offset, components }
        }
        DataArrayHeader::InlineAscii { components } => {
            let parsed_data =
                parse_ascii_inner_dataarray(reader, buffer, size_hint, expected_name)?;

            PartialDataArray::Parsed {
                buffer: parsed_data,
                components,
            }
        }
        DataArrayHeader::InlineBase64 { components } => {
            let parsed_data =
                parse_base64_inner_dataarray(reader, buffer, size_hint, expected_name)?;

            PartialDataArray::Parsed {
                buffer: parsed_data,
                components,
            }
        }
    };

    // if the element was not empty, then we need to close the element ourselves
    if !was_empty {
        println!("reading /DataArray ending element since this dataarray was not empty");
        // now we have to read the closing element for the dataarray
        // so we dont cause any trouble for future routines
        read_ending_element::<Mesh, _>(reader, buffer, "DataArray")?;
        println!("finished reading /DataArray ending element");
    }

    Ok(lazy_array)
}

/// cycle through buffers (and their offsets) and read the binary information from the
/// <AppendedBinary> section in order
pub fn read_appended_array_buffers<R: BufRead>(
    reader: &mut Reader<R>,
    buffer: &mut Vec<u8>,
    mut buffers: Vec<RefMut<'_, OffsetBuffer>>,
) -> Result<(), error::AppendedData> {
    // if we have any binary data:
    if buffers.len() > 0 {
        //we have some data to read - first organize all of the data by the offsets
        buffers.sort_unstable_by_key(|x| x.offset);

        dbg!("there are {} buffers", buffers.len());

        let mut iterator = buffers.iter_mut().peekable();

        clean_garbage_from_reader(reader, buffer)?;

        loop {
            if let Some(current_offset_buffer) = iterator.next() {
                // get the number of bytes to read based on the next element's offset
                let offset_length = iterator.peek().map(|offset_buffer| {
                    let diff = offset_buffer.offset - current_offset_buffer.offset;
                    diff as usize
                });

                let binary_length = current_offset_buffer.components
                    * current_offset_buffer.num_elements
                    * std::mem::size_of::<f64>();

                if let Some(calculated_offset_length) = offset_length {
                    assert_eq!(binary_length, calculated_offset_length);
                }

                crate::parse::parse_appended_binary(
                    reader,
                    buffer,
                    binary_length,
                    &mut current_offset_buffer.buffer,
                )?;
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
) -> Result<(bool, DataArrayHeader), Mesh> {
    // read the header for the element, it should have the element name `DataArray`
    let (was_empty, array_start) =
        read_empty_or_starting_element::<Mesh, _>(reader, buffer, "DataArray")?;
    dbg!(was_empty);

    let num_components =
        get_attribute_value::<Mesh>(&array_start, "NumberOfComponents", "DataArray")?;

    // TODO: use better error handling on this
    let components: usize = String::from_utf8(num_components.value.to_vec())
        .unwrap()
        .parse()
        .unwrap();

    let name = get_attribute_value::<Mesh>(&array_start, "Name", "DataArray")?;

    let format = get_attribute_value::<Mesh>(&array_start, "format", "DataArray")?;

    // TODO: better error handling on this
    assert_eq!(name.value, expected_name.as_bytes());

    let header = match format.value.as_ref() {
        b"appended" => {
            // appended binary data, we should have an extra `offset` attribute that we can read
            let offset = get_attribute_value::<Mesh>(&array_start, "offset", "DataArray")?;

            let offset_str = std::str::from_utf8(&offset.value).unwrap();
            // TODO: better error handling here
            let offset: i64 = offset_str.parse().expect(&format!(
                "data array offset `{}` coult not be parsed as integer",
                offset_str
            ));

            DataArrayHeader::AppendedBinary { offset, components }
        }
        b"binary" => {
            // we have base64 encoded data here
            DataArrayHeader::InlineBase64 { components }
        }
        b"ascii" => {
            // plain ascii data here
            DataArrayHeader::InlineAscii { components }
        }
        _ => {
            // TODO: find a better way to make errors here
            todo!()
        }
    };

    Ok((was_empty, header))
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
    pub fn new(partial: PartialDataArray, num_elements: usize) -> Self {
        match partial {
            PartialDataArray::Parsed { buffer, components } => {
                PartialDataArrayBuffered::Parsed { buffer, components }
            }
            PartialDataArray::AppendedBinary { offset, components } => {
                PartialDataArrayBuffered::AppendedBinary(RefCell::new(OffsetBuffer {
                    offset,
                    buffer: Vec::with_capacity(num_elements * components),
                    components,
                    num_elements,
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
    pub num_elements: usize,
}

impl Eq for OffsetBuffer {}

/// parse the values for a single inline ascii encoded array
///
/// ensure that before calling this function you have verified
/// that the data is base64 encoded with a call to `read_dataarray_header`
fn parse_ascii_inner_dataarray<'a, R: BufRead>(
    reader: &mut Reader<R>,
    buffer: &mut Vec<u8>,
    size_hint: usize,
    array_name: &str,
) -> Result<Vec<f64>, Mesh> {
    let event = read_body_element::<Mesh, _>(reader, buffer)?;
    let xml_bytes = event.into_inner();

    // TODO: better error handling here
    let location_data_string = std::str::from_utf8(&xml_bytes).unwrap();

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

    Ok(out)
}

/// parse the values for a single inline base64 encoded array
///
/// ensure that before calling this function you have verified
/// that the data is base64 encoded with a call to `read_dataarray_header`
fn parse_base64_inner_dataarray<'a, R: BufRead>(
    reader: &mut Reader<R>,
    buffer: &mut Vec<u8>,
    size_hint: usize,
    expected_name: &str,
) -> Result<Vec<f64>, Mesh> {
    let event = read_body_element::<Mesh, _>(reader, buffer)?;

    let base64_encoded_bytes = event.into_inner();
    let mut out = Vec::with_capacity(size_hint);

    let numerical_bytes =
        base64::decode(&base64_encoded_bytes).expect("could not decode base64 data array bytes");

    // normally we start with idx = 0, but since paraview expects the first 8 bytes
    // to be garbage information we need to skip the first 8 bytes before actually
    // reading the data
    let mut idx = 8;
    let inc = 8;

    // iterate through all the decoded base64 values (now in byte form), grabbing 8 bytes at a time
    // and convert them into floats
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

    Ok(out)
}

pub enum AppendedArrayLength {
    Known(usize),
    UntilEnd,
}

fn initialize_elements(buffer: &mut Vec<u8>, length: usize) {
    for _ in 0..length {
        buffer.push(0);
    }
}

fn ensure_buffer_length(buffer: &mut Vec<u8>, length: usize) {
    if buffer.len() < length {
        buffer.reserve(length - buffer.len());
        initialize_elements(buffer, length - buffer.len())
    }
}

pub fn clean_garbage_from_reader<R: BufRead>(
    reader: &mut Reader<R>,
    buffer: &mut Vec<u8>,
) -> Result<(), error::AppendedData> {
    // TODO:
    // previous parser used 16 bytes, why?
    //
    // 9 bytes of garbage to remove
    // 1 byte for the `_` character,
    // 8 filler bytes following that character
    let len = 9usize;

    // add extra 0 bytes to the buffer if required
    ensure_buffer_length(buffer, len);

    // pull the bytes manually from the internal reader
    let inner = reader.get_mut();
    inner.read_exact(&mut buffer[0..len]).unwrap();

    Ok(())
}

/// read information from the appended data binary buffer
pub fn parse_appended_binary<'a, R: BufRead>(
    reader: &mut Reader<R>,
    buffer: &mut Vec<u8>,
    length: usize,
    parsed_bytes: &mut Vec<f64>,
) -> Result<(), error::AppendedData> {
    ensure_buffer_length(buffer, length);

    let inner = reader.get_mut();
    inner
        .read_exact(&mut buffer.as_mut_slice()[0..length])
        .unwrap();

    let mut idx = 0;
    let inc = 8;

    while idx + inc <= length {
        if let Some(byte_slice) = buffer.get(idx..idx + inc) {
            let float = utils::bytes_to_float(byte_slice);
            parsed_bytes.push(float);
        }

        idx += inc;
    }

    Ok(())
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
            <Piece Extent="1 220 1 200 1 1">
            <Coordinates>
        "#;

        let mut reader = Reader::from_str(input);
        reader.trim_text(true);
        let mut buffer = Vec::new();

        let _ = read_to_grid_header(&mut reader, &mut buffer).unwrap();
        let whole_extent = read_rectilinear_header::<Spans3D, _>(&mut reader, &mut buffer).unwrap();
        let local_extent = read_to_coordinates::<Spans3D, _>(&mut reader, &mut buffer).unwrap();

        assert_eq!(
            whole_extent,
            Spans3D {
                x_start: 1,
                x_end: 220,
                y_start: 1,
                y_end: 200,
                z_start: 1,
                z_end: 1
            }
        );
        assert_eq!(
            local_extent,
            Spans3D {
                x_start: 1,
                x_end: 220,
                y_start: 1,
                y_end: 200,
                z_start: 1,
                z_end: 1
            }
        );
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
            </Coordinates>
            <PointData>
            "#;

        let mut reader = Reader::from_str(input);
        reader.trim_text(true);
        let mut buffer = Vec::new();

        let _local_extent: Spans3D = read_to_coordinates(&mut reader, &mut buffer).unwrap();
        let locations =
            crate::mesh::Mesh3DVisitor::read_headers(&spans, &mut reader, &mut buffer).unwrap();
        let out = locations.finish(&spans);

        prepare_reading_point_data(&mut reader, &mut buffer).unwrap();

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
        let mut reader = Reader::from_str(input);
        reader.trim_text(true);
        let mut buffer = Vec::new();

        let out = parse_dataarray_or_lazy(&mut reader, &mut buffer, "X", 4);
        dbg!(&out);
        let out = out.unwrap();
        let expected = 4;

        assert_eq!(out.unwrap_parsed().len(), expected);
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
    fn ascii_array_header() {
        let header = r#"<DataArray type="Float64" NumberOfComponents="1" Name="X" format="ascii">"#;

        let mut reader = Reader::from_str(header);
        reader.trim_text(true);
        let mut buffer = Vec::new();

        let out = read_dataarray_header(&mut reader, &mut buffer, "X");
        dbg!(&out);

        let array_type = out.unwrap().1;

        assert_eq!(array_type, DataArrayHeader::InlineAscii { components: 1 });
    }

    #[test]
    fn base64_array_header() {
        let header =
            r#"<DataArray type="Float64" NumberOfComponents="1" Name="X" format="binary">"#;
        let mut reader = Reader::from_str(header);
        reader.trim_text(true);
        let mut buffer = Vec::new();

        let out = read_dataarray_header(&mut reader, &mut buffer, "X");
        dbg!(&out);

        let array_type = out.unwrap().1;

        assert_eq!(array_type, DataArrayHeader::InlineBase64 { components: 1 });
    }

    #[test]
    fn appended_array_header() {
        let header = r#"<DataArray type="Float64" NumberOfComponents="3" Name="X" format="appended" offset="99">"#;

        let mut reader = Reader::from_str(header);
        reader.trim_text(true);
        let mut buffer = Vec::new();

        let out = read_dataarray_header(&mut reader, &mut buffer, "X");
        dbg!(&out);

        let array_type = out.unwrap().1;

        assert_eq!(
            array_type,
            DataArrayHeader::AppendedBinary {
                offset: 99,
                components: 3
            }
        );
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
        let mut reader = Reader::from_str(&string);
        reader.trim_text(true);
        let mut buffer = Vec::new();

        let parsed_result = parse_dataarray_or_lazy(&mut reader, &mut buffer, "X", 4);

        dbg!(&parsed_result);

        let out = parsed_result.unwrap();

        assert_eq!(out.unwrap_parsed(), &values);
    }

    #[test]
    fn appended_array() {
        let values = [1.0f64, 2.0, 3.0, 4.0];
        let values2 = [5.0f64, 6.0, 7.0, 8.0];

        dbg!(values[0].to_le_bytes());

        let mut output = Vec::new();
        let mut event_writer = crate::Writer::new_with_indent(&mut output, b'0', 4);

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

        //
        // we have to write some boilerplate so that the schema parses correctly
        //

        // close off the point data section
        let end_point_data = BytesEnd::new("PointData");
        event_writer
            .write_event(Event::End(end_point_data))
            .unwrap();

        // close off the piece
        let end_piece = BytesEnd::new("Piece");
        event_writer.write_event(Event::End(end_piece)).unwrap();

        // close off the RectilinearGrid element
        let end_grid = BytesEnd::new("RectilinearGrid");
        event_writer.write_event(Event::End(end_grid)).unwrap();

        // write the data inside the appended section
        crate::write_vtk::appended_binary_header_start(&mut event_writer).unwrap();

        /*
            current xml at this point is this

            <DataArray type="Float64" NumberOfComponents="1" Name="X" format="appended" offset="-8">
            </DataArray>
            <DataArray type="Float64" NumberOfComponents="1" Name="Y" format="appended" offset="24">
            </DataArray>
            </PointData>
            </Piece>
            </RectilinearGrid>
            <AppendedData encoding="raw">_
        */

        // need to write a 8 garbage bytes for things to work as expected - this
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

        //let x = String::from_utf8_lossy(&output);
        //println!("{x}");

        /*
                        <DataArray type="Float64" NumberOfComponents="1" Name="X" format="appended" offset="-8">
                            </DataArray>
                        <DataArray type="Float64" NumberOfComponents="1" Name="Y" format="appended" offset="24">
                            </DataArray>
                    </PointData>
                </Piece>
            </RectilinearGrid>
            <AppendedData encoding="raw">_BUNCH OF BINARY STUFF HERE</AppendedData>
        */

        let cursor = std::io::Cursor::new(output);
        let mut reader = Reader::from_reader(cursor);
        reader.trim_text(true);
        reader.check_end_names(false);
        let mut buffer = Vec::new();

        // write data array headers
        let parsed_header_1 = parse_dataarray_or_lazy(&mut reader, &mut buffer, "X", 4).unwrap();
        let parsed_header_2 = parse_dataarray_or_lazy(&mut reader, &mut buffer, "Y", 4).unwrap();

        let header_1 = parsed_header_1.unwrap_appended();
        let header_2 = parsed_header_2.unwrap_appended();

        let _ = close_element_to_appended_data(&mut reader, &mut buffer).unwrap();

        // open the AppendedData data element
        let _ = read_starting_element_with_name::<error::AppendedData, _>(
            &mut reader,
            &mut buffer,
            "AppendedData",
        )
        .unwrap();

        // remove the garbage bytes from the start of the VTK
        clean_garbage_from_reader(&mut reader, &mut buffer).unwrap();

        let len_1 = (header_2 - header_1) as usize;
        let len_2 = 4 * 8usize;

        let mut data_1 = Vec::new();
        let mut data_2 = Vec::new();

        parse_appended_binary(&mut reader, &mut buffer, len_1, &mut data_1).unwrap();
        parse_appended_binary(&mut reader, &mut buffer, len_2, &mut data_2).unwrap();

        assert_eq!(values.as_ref(), data_1);
        assert_eq!(values2.as_ref(), data_2);
    }
}
