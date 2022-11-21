use crate::prelude::*;

use quick_xml::events::attributes::Attribute;
use quick_xml::events::BytesEnd;
use quick_xml::events::BytesStart;
use quick_xml::events::Event;
use quick_xml::name::QName;

const STARTING_OFFSET: i64 = 0;

/// Write a given vtk file to a `Writer`
pub fn write_vtk<W, D, DOMAIN, EncMesh, EncArray>(
    writer: W,
    data: VtkData<DOMAIN, D>,
) -> Result<(), Error>
where
    W: Write,
    D: DataArray<EncArray>,
    DOMAIN: Domain<EncMesh>,
    EncArray: Encode,
    EncMesh: Encode,
{
    let mut writer = Writer::new(writer);

    //let version = xml::common::XmlVersion::Version10;

    let decl = quick_xml::events::BytesDecl::new("1.0", Some("UTF-8"), None);
    writer.write_event(Event::Decl(decl))?;

    let header = BytesStart::new("VTKFile").with_attributes(vec![
        make_att("type", "RectilinearGrid"),
        make_att("version", "1.0"),
        make_att("byte_order", "LittleEndian"),
        make_att("header_type", "UInt64"),
    ]);
    writer.write_event(Event::Start(header))?;

    // output the spans
    let span_str = data.domain.span_string();

    let grid = BytesStart::new("RectilinearGrid")
        .with_attributes(vec![make_att("WholeExtent", &span_str)]);
    writer.write_event(Event::Start(grid))?;

    //construct the basic framework for writing XML information
    let piece = BytesStart::new("Piece").with_attributes(vec![make_att("Extent", &span_str)]);
    writer.write_event(Event::Start(piece))?;

    let coordinates = BytesStart::new("Coordinates");
    writer.write_event(Event::Start(coordinates))?;

    // write the mesh information out
    data.domain.write_mesh_header(&mut writer)?;

    // either write the loation of all the verticies inline
    // here or write only the headers w/ offsets and write the data as binary later
    let starting_offset = if EncMesh::is_binary() {
        data.domain.mesh_bytes() as i64
    } else {
        STARTING_OFFSET
    };

    // close off the coordinates we opened
    let end_coordinates = BytesEnd::new("Coordinates");
    writer.write_event(Event::End(end_coordinates))?;

    // now, we have ended the coordinates we open the point data
    // section of the file
    let point_data = BytesStart::new("PointData");
    writer.write_event(Event::Start(point_data))?;

    // write the point data using the input data
    data.data.write_array_header(&mut writer, starting_offset)?;

    // close off the point data section
    let end_point_data = BytesEnd::new("PointData");
    writer.write_event(Event::End(end_point_data))?;

    // close off the piece
    let end_piece = BytesEnd::new("Piece");
    writer.write_event(Event::End(end_piece))?;

    // close off the RectilinearGrid element
    let end_grid = BytesEnd::new("RectilinearGrid");
    writer.write_event(Event::End(end_grid))?;

    // if we are doing _any_ sort of appending of data
    if EncMesh::is_binary() || EncArray::is_binary() {
        appended_binary_header_start(&mut writer)?;

        // for some reason paraview expects the first byte that is not '_' to
        // be garbage and it is skipped over. Previously we just used an initial offset=-8
        // to fix this issue, but it turns out that has unpredictable behavior when
        // writing appended binary coordinate arrays

        [100f64].as_ref().write_binary(&mut writer, false)?;

        // implementations will do nothing if they are not responsible for writing any binary
        // information
        data.domain.write_mesh_appended(&mut writer)?;
        // same here
        data.data.write_array_appended(&mut writer)?;

        appended_binary_header_end(&mut writer)?;
    }

    // end the vtk file
    let end_vtk = BytesEnd::new("VTKFile");
    writer.write_event(Event::End(end_vtk))?;

    Ok(())
}

pub(crate) fn appended_binary_header_start<W: Write>(
    writer: &mut Writer<W>,
) -> Result<(), quick_xml::Error> {
    let inner = writer.inner();
    inner.write_all(b"<AppendedData encoding=\"raw\">_")?;
    Ok(())
}

pub(crate) fn appended_binary_header_end<W: Write>(
    writer: &mut Writer<W>,
) -> Result<(), quick_xml::Error> {
    let inner = writer.inner();
    inner.write_all(b"</AppendedData>")?;
    Ok(())
}
/// the encoding to use when writing an inline dataarray
pub enum Encoding {
    Ascii,
    Base64,
}

impl Encoding {
    fn to_str(&self) -> &'static str {
        match &self {
            Self::Ascii => "ascii",
            Self::Base64 => "binary",
        }
    }
}

pub enum Precision {
    Float64,
    Float32,
}

impl Precision {
    fn to_str(&self) -> &'static str {
        match &self {
            Self::Float64 => "Float64",
            Self::Float32 => "Float32",
        }
    }
}

pub fn write_inline_array_header<W: Write>(
    writer: &mut Writer<W>,
    format: Encoding,
    name: &str,
    components: usize,
    precision: Precision,
) -> Result<(), Error> {
    let header = BytesStart::new("DataArray").with_attributes(vec![
        make_att("type", precision.to_str()),
        make_att("NumberOfComponents", &components.to_string()),
        make_att("Name", name),
        make_att("format", format.to_str()),
    ]);
    writer.write_event(Event::Start(header))?;

    Ok(())
}

pub fn close_inline_array_header<W: Write>(writer: &mut Writer<W>) -> Result<(), Error> {
    let header_end = BytesEnd::new("DataArray");
    writer.write_event(Event::End(header_end))?;

    Ok(())
}

/// write a single (inline) array of data (such as x-velocity)
/// to the vtk file.
pub fn write_inline_dataarray<W: Write, A: Array>(
    writer: &mut Writer<W>,
    data: &A,
    name: &str,
    encoding: Encoding,
) -> Result<(), Error> {
    match encoding {
        Encoding::Ascii => {
            data.write_ascii(writer, name)?;
        }
        Encoding::Base64 => {
            data.write_base64(writer, name)?;
        }
    };

    Ok(())
}

/// write the header for an appended data array that will later be written in the appended
/// section of the vtk.
///
/// if you call this function you are also responsible for calling
/// `write_appened_dataarray` with the data in the correct order
#[inline]
pub fn write_appended_dataarray_header<W: Write>(
    writer: &mut Writer<W>,
    name: &str,
    offset: i64,
    components: usize,
    precision: Precision,
) -> Result<(), Error> {
    let appended_header = BytesStart::new("DataArray").with_attributes(vec![
        make_att("type", precision.to_str()),
        make_att("NumberOfComponents", &components.to_string()),
        make_att("Name", name),
        make_att("format", "appended"),
        make_att("offset", &offset.to_string()),
    ]);

    let end_header = BytesEnd::new("DataArray");

    writer.write_event(Event::Start(appended_header))?;
    writer.write_event(Event::End(end_header))?;

    Ok(())
}

fn make_att<'a>(name: &'static str, value: &'a str) -> Attribute<'a> {
    let name = QName(name.as_bytes());
    Attribute {
        key: name,
        value: value.as_bytes().into(),
    }
}
