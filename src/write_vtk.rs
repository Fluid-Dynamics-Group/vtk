use super::data::VtkData;
use super::Array;
use super::DataArray;
use crate::Encode;
use crate::Error;
use crate::Mesh;

use std::borrow::Cow;
use std::io::Write;

use xml::attribute::Attribute;
use xml::name::Name;
use xml::namespace::Namespace;
use xml::writer::{EventWriter, XmlEvent};

const STARTING_OFFSET: i64 = 0;

/// Write a given vtk file to a `Writer`
pub fn write_vtk<W, D, MESH, EncMesh, EncArray>(
    writer: W,
    data: VtkData<MESH, D>,
) -> Result<(), Error>
where
    W: Write,
    D: DataArray<EncArray>,
    MESH: Mesh<EncMesh>,
    EncArray: Encode,
    EncMesh: Encode,
{
    let mut writer = EventWriter::new(writer);

    let version = xml::common::XmlVersion::Version10;
    writer.write(XmlEvent::StartDocument {
        version,
        encoding: None,
        standalone: None,
    })?;

    writer.write(XmlEvent::StartElement {
        name: Name::from("VTKFile"),
        attributes: vec![
            make_att("type", "RectilinearGrid"),
            make_att("version", "1.0"),
            make_att("byte_order", "LittleEndian"),
            make_att("header_type", "UInt64"),
        ]
        .into(),
        namespace: Cow::Owned(Namespace::empty()),
    })?;

    // output the spans
    let span_str = data.domain.span_string();

    writer.write(XmlEvent::StartElement {
        name: Name::from("RectilinearGrid"),
        attributes: vec![make_att("WholeExtent", &span_str)].into(),
        namespace: Cow::Owned(Namespace::empty()),
    })?;

    writer.write(XmlEvent::StartElement {
        name: Name::from("Piece"),
        attributes: vec![make_att("Extent", &span_str)].into(),
        namespace: Cow::Owned(Namespace::empty()),
    })?;

    writer.write(XmlEvent::StartElement {
        name: Name::from("Coordinates"),
        attributes: vec![].into(),
        namespace: Cow::Owned(Namespace::empty()),
    })?;

    // write the mesh information out
    data.domain.write_mesh_header(&mut writer)?;

    // either write the loation of all the verticies inline
    // here or write only the headers w/ offsets and write the data as binary later
    let starting_offset = if EncMesh::is_binary() {
        data.domain.mesh_bytes() as i64
    } else {
        STARTING_OFFSET
    };

    writer.write(XmlEvent::EndElement {
        name: Some(Name::from("Coordinates")),
    })?;

    writer.write(XmlEvent::StartElement {
        name: Name::from("PointData"),
        attributes: vec![].into(),
        namespace: Cow::Owned(Namespace::empty()),
    })?;

    data.data.write_array_header(&mut writer, starting_offset)?;

    writer.write(XmlEvent::EndElement {
        name: Some(Name::from("PointData")),
    })?;

    writer.write(XmlEvent::EndElement {
        name: Some(Name::from("Piece")),
    })?;
    writer.write(XmlEvent::EndElement {
        name: Some(Name::from("RectilinearGrid")),
    })?;

    // if we are doing _any_ sort of appending of data
    if EncMesh::is_binary() || EncArray::is_binary() {
        appended_binary_header_start(&mut writer)?;

        // for some reason paraview expects the first byte that is not '_' to
        // be garbage and it is skipped over. Previously we just used an initial offset=-8
        // to fix this issue, but it turns out that has unpredictable behavior when
        // writing :
        //      inline point location information + binary appended data (previously ok with
        //          offset)
        //      appended point loacation information + binary appended data (was failure)
        //      appended point location information + ascii data (was failure)
        //write_appended_dataarray(&mut writer, )?;

        [100f64].as_ref().write_binary(&mut writer)?;

        // implementations will do nothing if they are not responsible for writing any binary
        // information
        data.domain.write_mesh_appended(&mut writer)?;
        // same here
        data.data.write_array_appended(&mut writer)?;

        appended_binary_header_end(&mut writer)?;
    }

    writer.write(XmlEvent::EndElement {
        name: Some(Name::from("VTKFile")),
    })?;

    Ok(())
}

pub(crate) fn appended_binary_header_start<W: Write>(
    writer: &mut EventWriter<W>,
) -> Result<(), xml::writer::Error> {
    let inner = writer.inner_mut();
    inner.write_all(b"<AppendedData encoding=\"raw\">_")?;
    Ok(())
}

pub(crate) fn appended_binary_header_end<W: Write>(
    writer: &mut EventWriter<W>,
) -> Result<(), xml::writer::Error> {
    let inner = writer.inner_mut();
    inner.write_all(b"\n</AppendedData>")?;
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

pub fn write_inline_array_header<W: Write>(
    writer: &mut EventWriter<W>,
    format: Encoding,
    name: &str,
    components: usize,
) -> Result<(), Error> {
    writer.write(XmlEvent::StartElement {
        name: Name::from("DataArray"),
        attributes: vec![
            make_att("type", "Float64"),
            make_att("NumberOfComponents", &components.to_string()),
            make_att("Name", name),
            make_att("format", format.to_str()),
        ]
        .into(),
        namespace: Cow::Owned(Namespace::empty()),
    })?;

    Ok(())
}

pub fn close_inline_array_header<W: Write>(writer: &mut EventWriter<W>) -> Result<(), Error> {
    writer.write(XmlEvent::EndElement {
        name: Some(Name::from("DataArray")),
    })?;

    Ok(())
}

/// write a single (inline) array of data (such as x-velocity)
/// to the vtk file.
pub fn write_inline_dataarray<W: Write, A: Array>(
    writer: &mut EventWriter<W>,
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
    writer: &mut EventWriter<W>,
    name: &str,
    offset: i64,
    components: usize,
) -> Result<(), Error> {
    writer.write(XmlEvent::StartElement {
        name: Name::from("DataArray"),
        attributes: vec![
            make_att("type", "Float64"),
            make_att("NumberOfComponents", &components.to_string()),
            make_att("Name", name),
            make_att("format", "appended"),
            make_att("offset", &offset.to_string()),
        ]
        .into(),
        namespace: Cow::Owned(Namespace::empty()),
    })?;

    writer.write(XmlEvent::EndElement {
        name: Some(Name::from("DataArray")),
    })?;

    Ok(())
}

fn make_att<'a>(name: &'static str, value: &'a str) -> Attribute<'a> {
    let name = Name::from(name);
    Attribute::new(name, value)
}

//#[inline]
//fn ascii_coordinates_inline<W: Write>(
//    writer: &mut EventWriter<W>,
//    locations: &super::Locations,
//) -> Result<(), Error> {
//    write_inline_dataarray(writer, &locations.x_locations, "X", Encoding::Ascii)?;
//
//    write_inline_dataarray(writer, &locations.y_locations, "Y", Encoding::Ascii)?;
//
//    write_inline_dataarray(writer, &locations.z_locations, "Z", Encoding::Ascii)?;
//
//    Ok(())
//}
