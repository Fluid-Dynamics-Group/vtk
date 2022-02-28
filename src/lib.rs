#![doc = include_str!("../README.md")]

mod combine_vtk;
mod data;
mod array;
pub mod parse;
pub mod traits;
mod utils;
mod write_vtk;

pub(crate) use traits::{DataArray, ParseDataArray};

pub use combine_vtk::combine_vtk;
pub use data::{LocationSpans, Locations, VtkData};
pub use traits::{Array, FromBuffer};
pub use write_vtk::write_vtk;
pub use write_vtk::{write_appended_dataarray_header, write_inline_dataarray, Encoding};

pub use parse::read_and_parse as read_vtk;
pub use parse::ParseError;

#[cfg(feature = "derive")]
pub use vtk_derive::{DataArray, ParseDataArray};

#[cfg(feature = "derive")]
pub fn test_fn() {}

pub use xml::EventWriter;
pub use ndarray;

/// general purpose error enumeration for possible causes of failure.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("An io error occured: `{0}`")]
    Io(#[from] std::io::Error),
    #[error("The xml data inputted was malformed: `{0}`")]
    Xml(#[from] xml::reader::Error),
    #[error("Error when parsing the xml data: `{0}`")]
    Nom(#[from] parse::ParseError),
    #[error("Could not convert file to uf8 encoding: `{0}`")]
    Utf8(#[from] std::string::FromUtf8Error),
    #[error("Could not write XML data to file: `{0}`")]
    XmlWrite(#[from] xml::writer::Error),
}

#[cfg(test)]
mod helpers {
    use super::write_vtk::Encoding;
    use super::EventWriter;
    use std::io::Write;
    use std::ops::{Add, Div, Sub};

    #[derive(Debug, Clone, Default, derive_builder::Builder, PartialEq)]
    pub struct SpanData {
        pub u: Vec<f64>,
    }

    impl super::DataArray for SpanData {
        fn write_inline_dataarrays<W: Write>(
            &self,
            writer: &mut xml::EventWriter<W>,
        ) -> Result<(), crate::Error> {
            super::write_vtk::write_inline_dataarray(writer, &self.u, "u", Encoding::Ascii)?;
            Ok(())
        }

        fn is_appended_array() -> bool {
            false
        }

        fn write_appended_dataarray_headers<W: Write>(
            &self,
            writer: &mut EventWriter<W>,
            starting_offset: i64,
        ) -> Result<(), crate::Error> {
            Ok(())
        }
        fn write_appended_dataarrays<W: Write>(
            &self,
            writer: &mut EventWriter<W>,
        ) -> Result<(), crate::Error> {
            Ok(())
        }
    }

    impl super::ParseDataArray for SpanData {
        fn parse_dataarrays(
            rest: &[u8],
            span_info: &crate::LocationSpans,
            partial: crate::parse::LocationsPartial,
        ) -> Result<(Self, crate::Locations), crate::parse::ParseError> {
            let (rest, u) = crate::parse::parse_dataarray_or_lazy(rest, b"u", 1000)?;
            let locations = crate::Locations {
                x_locations: partial.x.unwrap_parsed(),
                y_locations: partial.y.unwrap_parsed(),
                z_locations: partial.z.unwrap_parsed(),
            };
            Ok((
                Self {
                    u: u.unwrap_parsed(),
                },
                locations,
            ))
        }
    }

    impl Add for SpanData {
        type Output = Self;

        fn add(mut self, other: Self) -> Self {
            self.u
                .iter_mut()
                .zip(other.u.into_iter())
                .for_each(|(s, o)| *s = *s + o);
            self
        }
    }

    impl Div<f64> for SpanData {
        type Output = Self;

        fn div(mut self, other: f64) -> Self::Output {
            self.u.iter_mut().for_each(|s| *s = *s / other);
            self
        }
    }

    impl Sub for SpanData {
        type Output = Self;

        fn sub(mut self, other: Self) -> Self {
            self.u
                .iter_mut()
                .zip(other.u.into_iter())
                .for_each(|(s, o)| *s = *s - o);
            self
        }
    }

    #[derive(Debug, Clone)]
    pub struct DataItem {
        pub(crate) data: crate::VtkData<SpanData>,
        pub(crate) proc_number: usize,
        pub(crate) step_number: usize,
    }

    #[rustfmt::skip]
    impl crate::traits::Combine for Vec<DataItem> {
        fn total_procs(&self) -> usize {
            self.len()
        }
        fn x_dims(&self) -> (usize, usize) {
            let start = self.into_iter().min_by_key(|x| x.proc_number).unwrap().data.spans.x_start;
            let end = self.into_iter().max_by_key(|x| x.proc_number).unwrap().data.spans.x_end;
            (start, end)
        }
        fn y_dims(&self) -> (usize, usize) {
            let start = self.into_iter().min_by_key(|x| x.proc_number).unwrap().data.spans.y_start;
            let end = self.into_iter().max_by_key(|x| x.proc_number).unwrap().data.spans.y_end;
            (start, end)
        }
        fn z_dims(&self) -> (usize, usize) {
            let start = self.into_iter().min_by_key(|x| x.proc_number).unwrap().data.spans.y_start;
            let end = self.into_iter().max_by_key(|x| x.proc_number).unwrap().data.spans.y_end;
            (start, end)
        }
        fn x_locations(&self) -> Vec<f64> {
            let mut out = Vec::with_capacity(self.len() * self[0].data.locations.x_locations.len());
            self.into_iter().for_each(|item| out.extend(&item.data.locations.x_locations));
            out
        }
        fn y_locations(&self) -> Vec<f64> {
            let mut out = Vec::with_capacity(self.len() * self[0].data.locations.y_locations.len());
            self.into_iter().for_each(|item| out.extend(&item.data.locations.y_locations));
            out
        }
        fn z_locations(&self) -> Vec<f64> {
            let mut out = Vec::with_capacity(self.len() * self[0].data.locations.z_locations.len());
            self.into_iter().for_each(|item| out.extend(&item.data.locations.z_locations));
            out
        } 
    }

    impl From<Vec<DataItem>> for SpanData {
        fn from(mut x: Vec<DataItem>) -> SpanData {
            x.sort_unstable_by_key(|x| x.proc_number);
            let u = x.into_iter().map(|x| x.data.data.u).flatten().collect();
            SpanData { u }
        }
    }

    #[derive(Debug)]
    pub(crate) struct SpanDataBinary {
        pub u: Vec<f64>,
        pub v: Vec<f64>,
        pub w: Vec<f64>,
    }

    impl crate::traits::ParseDataArray for SpanDataBinary {
        fn parse_dataarrays(
            data: &[u8],
            span_info: &super::LocationSpans,
            locations: super::parse::LocationsPartial,
        ) -> Result<(Self, super::Locations), super::parse::ParseError> {
            let mut binary_info: Vec<&mut crate::parse::OffsetBuffer> = Vec::new();
            //
            let len = span_info.x_len() * span_info.y_len() * span_info.z_len();
            let (data, u) = crate::parse::parse_dataarray_or_lazy(data, b"u", len)?;
            let (data, v) = crate::parse::parse_dataarray_or_lazy(data, b"v", len)?;
            let (data, w) = crate::parse::parse_dataarray_or_lazy(data, b"w", len)?;

            let mut locations_x__ = crate::parse::PartialDataArrayBuffered::new(locations.x, len);
            let mut locations_y__ = crate::parse::PartialDataArrayBuffered::new(locations.y, len);
            let mut locations_z__ = crate::parse::PartialDataArrayBuffered::new(locations.z, len);

            let mut u = crate::parse::PartialDataArrayBuffered::new(u, len);
            let mut v = crate::parse::PartialDataArrayBuffered::new(v, len);
            let mut w = crate::parse::PartialDataArrayBuffered::new(w, len);

            // push into the arryas
            match &mut locations_x__ {
                crate::parse::PartialDataArrayBuffered::AppendedBinary(offset) => {
                    binary_info.push(offset)
                }
                _ => (),
            };

            match &mut locations_y__ {
                crate::parse::PartialDataArrayBuffered::AppendedBinary(offset) => {
                    binary_info.push(offset)
                }
                _ => (),
            };

            match &mut locations_z__ {
                crate::parse::PartialDataArrayBuffered::AppendedBinary(offset) => {
                    binary_info.push(offset)
                }
                _ => (),
            };

            match &mut u {
                crate::parse::PartialDataArrayBuffered::AppendedBinary(offset) => {
                    binary_info.push(offset)
                }
                _ => (),
            };

            match &mut v {
                crate::parse::PartialDataArrayBuffered::AppendedBinary(offset) => {
                    binary_info.push(offset)
                }
                _ => (),
            };

            match &mut w {
                crate::parse::PartialDataArrayBuffered::AppendedBinary(offset) => {
                    binary_info.push(offset)
                }
                _ => (),
            };

            // if we have any binary data:
            if binary_info.len() > 0 {
                //we have some data to read - first organize all of the data by the offsets
                binary_info.sort_unstable();

                let mut iterator = binary_info.iter_mut().peekable();
                let (mut appended_data, _) = crate::parse::setup_appended_read(data)?;

                loop {
                    if let Some(current_offset_buffer) = iterator.next() {
                        // get the number of bytes to read based on the next element's offset
                        let reading_offset = iterator
                            .peek()
                            .map(|offset_buffer| {
                                crate::parse::AppendedArrayLength::Known(
                                    (offset_buffer.offset - current_offset_buffer.offset) as usize,
                                )
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

            let locations = crate::Locations {
                x_locations: locations_x__.into_buffer(),
                y_locations: locations_y__.into_buffer(),
                z_locations: locations_z__.into_buffer(),
            };

            let u = u.into_buffer();
            let v = v.into_buffer();
            let w = w.into_buffer();

            Ok((Self { u, v, w }, locations))
        }
    }
}

#[cfg(all(test, feature = "derive"))]
mod parsing_writing_compare {
    use crate as vtk;

    #[derive(super::ParseDataArray, super::DataArray, Clone, Debug)]
    #[vtk(encoding = "binary")]
    struct Binary {
        rho: Vec<f64>,
        u: Vec<f64>,
        v: Vec<f64>,
        w: Vec<f64>,
    }

    #[derive(vtk::ParseDataArray, vtk::DataArray, Clone)]
    #[vtk(encoding = "base64")]
    struct Base64 {
        rho: Vec<f64>,
        u: Vec<f64>,
        v: Vec<f64>,
        w: Vec<f64>,
    }

    impl From<Binary> for Base64 {
        fn from(x: Binary) -> Self {
            let Binary { rho, u, v, w } = x;
            Base64 { rho, u, v, w }
        }
    }

    fn create_data() -> super::VtkData<Binary> {
        let locations = super::Locations {
            x_locations: vec![0., 1., 2., 3., 4.],
            y_locations: vec![0., 1., 2., 3., 4.],
            z_locations: vec![0., 1., 2., 3., 4.],
        };

        let spans = super::LocationSpans {
            x_start: 0,
            x_end: 4,
            y_start: 0,
            y_end: 4,
            z_start: 0,
            z_end: 4,
        };

        let length = spans.x_len() * spans.y_len() * spans.z_len();

        let rho: Vec<_> = std::iter::repeat(0)
            .take(length)
            .enumerate()
            .map(|(i, _)| i as f64)
            .collect();
        let u = std::iter::repeat(0)
            .take(length)
            .enumerate()
            .map(|(i, _)| i as f64)
            .collect();
        let v = std::iter::repeat(0)
            .take(length)
            .enumerate()
            .map(|(i, _)| i as f64)
            .collect();
        let w = std::iter::repeat(0)
            .take(length)
            .enumerate()
            .map(|(i, _)| i as f64)
            .collect();

        dbg!(rho.len());

        let data = Binary { rho, u, v, w };

        let data = super::VtkData {
            locations,
            spans,
            data,
        };

        data
    }

    #[test]
    fn inline_ascii_points_appended_binary_data() {
        let data = create_data();
        let mut writer = Vec::new();
        vtk::write_vtk(&mut writer, data.clone(), false).unwrap();

        let output_data: vtk::VtkData<Binary> =
            vtk::parse::parse_xml_document(writer.as_slice()).unwrap();

        assert_eq!(output_data.spans, output_data.spans);
        assert_eq!(output_data.locations, output_data.locations);
        assert_eq!(output_data.data.rho, data.data.rho);
        assert_eq!(output_data.data.u, data.data.u);
        assert_eq!(output_data.data.v, data.data.v);
        assert_eq!(output_data.data.w, data.data.w);
    }

    #[test]
    fn appended_ascii_points_appended_binary_data() {
        let data = create_data();
        let mut writer = Vec::new();
        vtk::write_vtk(&mut writer, data.clone(), true).unwrap();

        let output_data: vtk::VtkData<Binary> =
            vtk::parse::parse_xml_document(writer.as_slice()).unwrap();

        assert_eq!(output_data.spans, output_data.spans);
        assert_eq!(output_data.locations, output_data.locations);
        assert_eq!(output_data.data.rho, data.data.rho);
        assert_eq!(output_data.data.u, data.data.u);
        assert_eq!(output_data.data.v, data.data.v);
        assert_eq!(output_data.data.w, data.data.w);
    }

    #[test]
    fn inline_points_inline_base64() {
        let data = create_data();
        let mut writer = Vec::new();

        let locations = data.locations.clone();
        let spans = data.spans.clone();
        let data = data.data.clone();

        let base64 = vtk::VtkData {
            locations: locations.clone(),
            spans: spans.clone(),
            data: Base64::from(data.clone()),
        };

        vtk::write_vtk(&mut writer, base64.clone(), true).unwrap();

        let output_data: vtk::VtkData<Base64> =
            vtk::parse::parse_xml_document(writer.as_slice()).unwrap();

        assert_eq!(output_data.spans, output_data.spans);
        assert_eq!(output_data.locations, output_data.locations);
        assert_eq!(output_data.data.rho, base64.data.rho);
        assert_eq!(output_data.data.u, base64.data.u);
        assert_eq!(output_data.data.v, base64.data.v);
        assert_eq!(output_data.data.w, base64.data.w);
    }
}
