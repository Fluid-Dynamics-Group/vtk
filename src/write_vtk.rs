use super::data::VtkData;
use super::DataArray;
use crate::Error;

use std::borrow::Cow;
use std::io::Write;

use xml::attribute::Attribute;
use xml::name::Name;
use xml::namespace::Namespace;
use xml::writer::{EventWriter, XmlEvent};

/// Write a given vtk file to a `Writer`
pub fn write_vtk<W: Write, D: DataArray>(
    writer: W,
    data: VtkData<D>,
    append_coordinates: bool,
) -> Result<(), Error> {
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

    let span_str = data.spans.to_string();
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

    // either write the loation of all the verticies inline
    // here or write only the headers w/ offsets and write the data as binary later
    let starting_offset = if !append_coordinates {
        ascii_coordinates_inline(&mut writer, &data.locations)?;
        // this is some funky hack to includethe first value in the data array that
        // paraview somehow skips over - not sure if this will ever be fixed on their end
        -8
    } else {
        // write the headers now with the intention to write the full dataarrays later
        // in the appended section. return the total offset required before writing
        // any additional data arrays
        coordinate_headers_appended(&mut writer, &data.locations)?
    };

    writer.write(XmlEvent::EndElement {
        name: Some(Name::from("Coordinates")),
    })?;

    writer.write(XmlEvent::StartElement {
        name: Name::from("PointData"),
        attributes: vec![].into(),
        namespace: Cow::Owned(Namespace::empty()),
    })?;

    // inline dataarray declarations
    if !D::is_appended_array() {
        // call the data element of VtkData to write itself out
        data.data.write_inline_dataarrays(&mut writer)?;
    } else {
        data.data
            .write_appended_dataarray_headers(&mut writer, starting_offset)?;
    }

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
    if append_coordinates || D::is_appended_array() {
        appended_binary_header_start(&mut writer)?;

        // write the appended point data here if required
        if append_coordinates {
            appended_coordinate_dataarrays(&mut writer, &data.locations)?;
        }

        // write the appended flow data here
        if D::is_appended_array() {
            data.data.write_appended_dataarrays(&mut writer)?;
        }

        appended_binary_header_end(&mut writer)?;
    }

    writer.write(XmlEvent::EndElement {
        name: Some(Name::from("VTKFile")),
    })?;

    Ok(())
}

pub(crate) fn appended_binary_header_start<W: Write>(writer: &mut EventWriter<W>) -> Result<(), xml::writer::Error> {
    let inner = writer.inner_mut();
    inner.write_all(b"<AppendedData encoding=\"raw\">_")?;
    Ok(())
}

pub(crate) fn appended_binary_header_end<W: Write>(writer: &mut EventWriter<W>) -> Result<(), xml::writer::Error> {
    let inner = writer.inner_mut();
    inner.write_all(b"\n</AppendedData>")?;
    Ok(())
}
/// the encoding to use when writing an inline dataarray
pub enum Encoding {
    Ascii,
    Base64,
}

/// write a single (inline) array of data (such as x-velocity)
/// to the vtk file.
pub fn write_inline_dataarray<W: Write>(
    writer: &mut EventWriter<W>,
    data: &[f64],
    name: &str,
    encoding: Encoding,
) -> Result<(), Error> {
    let data = match encoding {
        Encoding::Ascii => {
            writer.write(XmlEvent::StartElement {
                name: Name::from("DataArray"),
                attributes: vec![
                    make_att("type", "Float64"),
                    make_att("NumberOfComponents", "1"),
                    make_att("Name", name),
                    make_att("format", "ascii"),
                ]
                .into(),
                namespace: Cow::Owned(Namespace::empty()),
            })?;

            // write out all numbers with 12 points of precision
            data.into_iter()
                .map(|x| {
                    let mut buffer = ryu::Buffer::new();
                    let mut num = buffer.format(*x).to_string();
                    num.push(' ');
                    num
                })
                .collect()
        }
        Encoding::Base64 => {
            writer.write(XmlEvent::StartElement {
                name: Name::from("DataArray"),
                attributes: vec![
                    make_att("type", "Float64"),
                    make_att("NumberOfComponents", "1"),
                    make_att("Name", name),
                    make_att("format", "binary"),
                ]
                .into(),
                namespace: Cow::Owned(Namespace::empty()),
            })?;

            // convert the floats into LE bytes
            let mut byte_data = Vec::with_capacity(data.len() * 8);
            data.into_iter()
                .for_each(|float| byte_data.extend_from_slice(&float.to_le_bytes()));

            base64::encode(byte_data.as_slice())
        }
    };

    writer.write(XmlEvent::Characters(&data))?;

    writer.write(XmlEvent::EndElement {
        name: Some(Name::from("DataArray")),
    })?;

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
) -> Result<(), Error> {
    writer.write(XmlEvent::StartElement {
        name: Name::from("DataArray"),
        attributes: vec![
            make_att("type", "Float64"),
            make_att("NumberOfComponents", "1"),
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

/// write the file data to the file to the appended section in binary form
///
/// You must ensure that you have called `write_appended_dataarray_header` with
/// the correct offset before calling this function.
pub fn write_appended_dataarray<W: Write>(
    writer: &mut EventWriter<W>,
    data: &[f64],
) -> Result<(), Error> {
    let writer = writer.inner_mut();
    let mut bytes = Vec::with_capacity(data.len() * 8);

    data.into_iter()
        .for_each(|float| bytes.extend(float.to_le_bytes()));

    writer.write_all(&bytes)?;

    Ok(())
}

fn make_att<'a>(name: &'static str, value: &'a str) -> Attribute<'a> {
    let name = Name::from(name);
    Attribute::new(name, value)
}

#[inline]
fn ascii_coordinates_inline<W: Write>(
    writer: &mut EventWriter<W>,
    locations: &super::Locations,
) -> Result<(), Error> {
    write_inline_dataarray(writer, &locations.x_locations, "X", Encoding::Ascii)?;

    write_inline_dataarray(writer, &locations.y_locations, "Y", Encoding::Ascii)?;

    write_inline_dataarray(writer, &locations.z_locations, "Z", Encoding::Ascii)?;

    Ok(())
}

/// write the headers for the coordinates assuming that we are going to write
/// the raw data for the headers in the appended section later
///
/// does not write the data inline
#[inline]
fn coordinate_headers_appended<W: Write>(
    writer: &mut EventWriter<W>,
    locations: &super::Locations,
) -> Result<i64, Error> {
    let mut offset = -8;

    write_appended_dataarray_header(writer, "X", offset)?;
    offset += (std::mem::size_of::<f64>() * locations.x_locations.len()) as i64;

    write_appended_dataarray_header(writer, "Y", offset)?;
    offset += (std::mem::size_of::<f64>() * locations.y_locations.len()) as i64;

    write_appended_dataarray_header(writer, "Z", offset)?;
    offset += (std::mem::size_of::<f64>() * locations.z_locations.len()) as i64;

    Ok(offset)
}

/// Write the raw data for the DataArray's used for the coordinates
///
/// this is only here to make the main `write_vtk` function more readable
#[inline]
fn appended_coordinate_dataarrays<W: Write>(
    writer: &mut EventWriter<W>,
    locations: &super::Locations,
) -> Result<(), Error> {
    write_appended_dataarray(writer, &locations.x_locations)?;
    write_appended_dataarray(writer, &locations.y_locations)?;
    write_appended_dataarray(writer, &locations.z_locations)?;

    Ok(())
}
