use crate::prelude::*;

use super::event_summary::EventSummary;

use quick_xml::events::attributes::Attribute;
use quick_xml::events::BytesEnd;
use quick_xml::events::BytesStart;
use quick_xml::events::BytesText;
use quick_xml::events::Event;
use quick_xml::events;
use quick_xml::name::QName;
use quick_xml::reader::Reader;

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
    #[error("Error parsing vtk file before coordinate section: {0}")]
    PreparePointData(PreparePointData),
    #[error("Error parsing vtk file before coordinate section: {0}")]
    CloseElements(CloseElements),
    #[error("Error parsing vtk file before coordinate section: {0}")]
    AppendedData(AppendedData),
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
    xml_err: quick_xml::Error,
}

#[derive(From, Display, Debug)]
#[display(fmt = "failed to parse an xml attribute: {att_err}")]
pub struct MalformedAttribute {
    att_err: quick_xml::events::attributes::AttrError,
}

#[derive(From, Display, Debug)]
#[display(
    fmt = "unexpected element. Expected `{expected_name}`, got {actual_element}"
)]
pub struct UnexpectedElement {
    expected_name: String,
    actual_element: EventSummary,
}

impl UnexpectedElement {
    pub(crate) fn new<T: Into<String>>(expected_name: T, actual_element: EventSummary) -> Self {
        Self {
            expected_name: expected_name.into(),
            actual_element
        }
    }
}

#[derive(From, Display, Debug, Constructor)]
#[display(
    fmt = "unexpected attribute value for {attribute_name} in {element_name} element: expected {expected_value}, got {actual_value}"
)]
pub struct UnexpectedAttributeValue {
    pub(crate) element_name: String,
    pub(crate) attribute_name: String,
    pub(crate) expected_value: String,
    pub(crate) actual_value: ParsedNameOrBytes,
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
    Bytes(Vec<u8>),
}

impl ParsedNameOrBytes {
    fn new(bytes: &[u8]) -> Self {
        let vec = Vec::from(bytes);
        match String::from_utf8(vec) {
            Ok(string) => Self::Utf8(string),
            Err(e) => Self::Bytes(e.into_bytes()),
        }
    }
}

impl<'a> From<QName<'a>> for ParsedNameOrBytes {
    fn from(x: QName) -> Self {
        Self::new(x.as_ref())
    }
}

impl<'a> From<std::borrow::Cow<'a, [u8]>> for ParsedNameOrBytes {
    fn from(x: std::borrow::Cow<'a, [u8]>) -> Self {
        Self::new(x.as_ref())
    }
}

impl<'a> From<&'a str> for ParsedNameOrBytes {
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
    #[error("{0}")]
    InlineAsciiArray(InlineAsciiArray),
}

#[derive(Debug, thiserror::Error, From)]
pub enum PreparePointData {
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
    #[error("{0}")]
    InlineAsciiArray(InlineAsciiArray),
}

#[derive(Debug, thiserror::Error, From)]
pub enum CloseElements {
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
    #[error("{0}")]
    InlineAsciiArray(InlineAsciiArray),
}

#[derive(Debug, thiserror::Error, From)]
pub enum AppendedData {
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
    #[error("{0}")]
    InlineAsciiArray(InlineAsciiArray),
    #[error("{0}")]
    ParsingBinary(ParsingBinary),
}

#[derive(Debug, thiserror::Error, From)]
pub enum ParsingBinary {
    #[error("removing the leading 16 bytes from the <AppendedData> element caused an error")]
    LeadingBytes,
    #[error("Failed to slices data array from appended binary bytes. Appended binary section may be too short")]
    BinaryToFloat,
}

#[derive(From, Display, Debug, Constructor)]
#[display(fmt = "Failed to parse inline ascii array `{array_name}` in DataArray element")]
pub struct InlineAsciiArray {
    array_name: String,
}
