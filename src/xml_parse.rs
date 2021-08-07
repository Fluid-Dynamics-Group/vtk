use super::data::{LocationSpans, Locations, VtkData};
use super::ParseDataArray;
use crate::Error;

use std::io::Read;

use nom::bytes::complete::{tag, take_till, take_until};
use nom::sequence::tuple;
use nom::IResult;

use std::fmt;

type NomErr<'a> = nom::Err<nom::error::Error<&'a str>>;

/// An error caused from parsing the vtk files
#[derive(Debug, thiserror::Error)]
pub struct ParseError {
    nom_reason: String,
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
            nom_reason: error.input.to_string(),
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
        write!(
            f,
            "reason:{} \tnom_reason:{} \t errorcode:{:?}",
            self.extra_info, self.nom_reason, self.nom_code
        )
    }
}

/// read in and parse an entire vtk file for a given path
pub fn read_and_parse<D: ParseDataArray>(path: &std::path::Path) -> Result<VtkData<D>, Error> {
    let mut file = std::fs::File::open(path)?;
    let mut buffer = Vec::with_capacity(1024 * 1024 * 3);
    file.read_to_end(&mut buffer)?;

    let string = String::from_utf8(buffer)?;
    parse_xml_document(&string)
}

pub(crate) fn parse_xml_document<D: ParseDataArray>(i: &str) -> Result<VtkData<D>, Error> {
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

    let data = D::parse_dataarrays(rest_of_document, &spans)?;

    Ok(VtkData {
        spans,
        locations,
        data,
    })
}

pub(crate) fn find_extent(i: &str) -> IResult<&str, LocationSpans> {
    let (start_extent, _xml_header_info) = take_until("WholeExtent")(i)?;
    let (extent_string_start, _whole_extent_header) = tag("WholeExtent=\"")(start_extent)?;
    let (rest_of_document, extent_string) = take_till(|c| c == '\"')(extent_string_start)?;

    let spans = LocationSpans::new(extent_string);

    Ok((rest_of_document, spans))
}

pub(crate) fn parse_locations<'a>(
    i: &'a str,
    span_info: &LocationSpans,
) -> IResult<&'a str, Locations> {
    let (rest, x_locations) = parse_dataarray(i, "X", span_info.x_len())?;
    let (rest, y_locations) = parse_dataarray(rest, "Y", span_info.y_len())?;
    let (rest, z_locations) = parse_dataarray(rest, "Z", span_info.z_len())?;

    let locations = Locations {
        x_locations,
        y_locations,
        z_locations,
    };

    Ok((rest, locations))
}

/// parse the the values for a single inline DataArray.
///
/// Most useful in a `traits::ParseDataArray`
/// implementation.
///
/// ### `xml_bytes`
///
/// is the string slice that starts with the dataarray information. This bytes slice
/// may contain more information after the dataarray, which is returned from this function
///
/// ### `exptected_data`
///
/// is the name of the field that you expect to be present for that dataarray
///
/// ### `size_hint`
///
/// is the numbner of elements that will be pre-allocated to a vector. If you have an
/// estimate of the approximate size of the data use this value to provide a small
/// optimization.
///
/// ## Returns
///
/// A tuple of the remaining data in the string (not parsed) and the floating point data that
/// was contained in the DataArray
pub fn parse_dataarray<'a>(
    xml_bytes: &'a str,
    expected_data: &str,
    size_hint: usize,
) -> IResult<&'a str, Vec<f64>> {
    let (rest, _unusable) = take_until("<DataArray")(xml_bytes)?;
    // TODO: update streams code so that we can just write Name here. The z data has a lowercase N
    let (name_start, _unusable) = take_until("ame=")(rest)?;
    let (name_start, _unusable) = tag("ame=\"")(name_start)?;
    let (rest, name) = take_till(|c| c == '\"')(name_start)?;

    assert_eq!(name, expected_data);

    let (brace_then_location_data, _unusable) = take_till(|c| c == '>')(rest)?;
    let (location_data_and_rest, _whitespace) = tuple((
        tag(">"),
        take_till(|c: char| c.is_ascii_digit() || c == '.' || c == '-'),
    ))(brace_then_location_data)?;
    let (rest_of_document, location_data) = take_till(|c| c == '<')(location_data_and_rest)?;

    let mut out = Vec::with_capacity(size_hint);

    location_data
        .trim_end()
        .split_ascii_whitespace()
        .for_each(|x| {
            let num = x
                .parse()
                .expect("character data could not be parsed into a number");
            out.push(num);
        });

    Ok((rest_of_document, out))
}

//fn parse_fluid_data<'a>(i: &'a str, span_info: &LocationSpans) -> IResult<&'a str, SpanData> {
//
//    Ok((rest, data))
//}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn shred_to_extent() {
        let input = r#"<VTKFile type="RectilinearGrid" version="1.0" byte_order="LittleEndian" header_type="UInt64">
            <RectilinearGrid WholeExtent="1 220 1 200 1 1">
            <Piece Extent="1 220 1 200 1 1">"#;
        let out = find_extent(input);
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
        let out = parse_locations(input, &locations);
        let out = out.unwrap().1;
        assert_eq!(out.x_locations.len(), 4);
        assert_eq!(out.y_locations.len(), 4);
        assert_eq!(out.z_locations.len(), 4);
    }

    #[test]
    fn single_loation() {
        let input = r#"
                <DataArray type="Float64" NumberOfComponents="1" Name="X" format="ascii">
                    .0000000000E+00 .3981797497E-01 .7963594994E-01 .1194539249E+00
                </DataArray>
            "#;
        let out = parse_dataarray(input, "X", 4);
        let out = out.unwrap();
        let expected = 4;

        assert_eq!(out.1.len(), expected);
    }

    #[test]
    fn full_vtk() {
        let out = read_and_parse(std::path::Path::new("./static/sample_vtk_file.vtk"));
        dbg!(&out);
        let out: crate::VtkData<crate::helpers::SpanData> = out.unwrap();
    }
}
