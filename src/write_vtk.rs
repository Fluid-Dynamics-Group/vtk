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
pub fn write_vtk<W: Write, D: DataArray>(writer: W, data: VtkData<D>) -> Result<(), Error> {
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

    write_dataarray(&mut writer, &data.locations.x_locations, "X", true)?;
    write_dataarray(&mut writer, &data.locations.y_locations, "Y", true)?;
    write_dataarray(&mut writer, &data.locations.z_locations, "Z", true)?;

    writer.write(XmlEvent::EndElement {
        name: Some(Name::from("Coordinates")),
    })?;

    writer.write(XmlEvent::StartElement {
        name: Name::from("PointData"),
        attributes: vec![].into(),
        namespace: Cow::Owned(Namespace::empty()),
    })?;

    // call the data element of VtkData to write itself out
    data.data.write_dataarray(&mut writer)?;

    writer.write(XmlEvent::EndElement {
        name: Some(Name::from("PointData")),
    })?;

    writer.write(XmlEvent::EndElement {
        name: Some(Name::from("Piece")),
    })?;
    writer.write(XmlEvent::EndElement {
        name: Some(Name::from("RectilinearGrid")),
    })?;
    writer.write(XmlEvent::EndElement {
        name: Some(Name::from("VTKFile")),
    })?;

    Ok(())
}

pub fn write_dataarray<W: Write>(
    writer: &mut EventWriter<W>,
    data: &[f64],
    name: &str,
    make_ascii: bool,
) -> Result<(), Error> {
    let data = if make_ascii {
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
    } else {
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
    };

    writer.write(XmlEvent::Characters(&data))?;

    writer.write(XmlEvent::EndElement {
        name: Some(Name::from("DataArray")),
    })?;

    Ok(())
}

fn make_att<'a>(name: &'static str, value: &'a str) -> Attribute<'a> {
    let name = Name::from(name);
    Attribute::new(name, value)
}
