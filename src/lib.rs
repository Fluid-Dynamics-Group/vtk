#![doc = include_str!("../README.md")]

mod combine_vtk;
mod data;
mod iter;
pub mod traits;
mod utils;
mod write_vtk;
pub mod xml_parse;

pub use xml_parse as parse;

pub(crate) use traits::{DataArray, ParseDataArray};

pub use combine_vtk::combine_vtk;
pub use data::{LocationSpans, Locations, VtkData};
pub use write_vtk::write_vtk;
pub use write_vtk::{
    write_appended_dataarray, write_appended_dataarray_header, write_inline_dataarray, Encoding,
};

pub use xml_parse::read_and_parse as read_vtk;
pub use xml_parse::ParseError;

#[cfg(feature = "derive")]
pub use vtk_derive::{DataArray, ParseDataArray};

pub use xml::EventWriter;

/// general purpose error enumeration for possible causes of failure.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("An io error occured: `{0}`")]
    Io(#[from] std::io::Error),
    #[error("The xml data inputted was malformed: `{0}`")]
    Xml(#[from] xml::reader::Error),
    #[error("Error when parsing the xml data: `{0}`")]
    Nom(#[from] xml_parse::ParseError),
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
        pub rho: Vec<f64>,
    }

    impl super::DataArray for SpanData {
        fn write_inline_dataarrays<W: Write>(
            &self,
            writer: &mut xml::EventWriter<W>,
        ) -> Result<(), crate::Error> {
            super::write_vtk::write_inline_dataarray(writer, &self.rho, "rho", Encoding::Ascii)?;
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
            partial: crate::xml_parse::LocationsPartial,
        ) -> Result<(Self, crate::Locations), crate::xml_parse::ParseError> {
            let (rest, rho) = crate::xml_parse::parse_dataarray_or_lazy(rest, b"rho", 1000)?;
            let locations = crate::Locations {
                x_locations: partial.x.unwrap_parsed(),
                y_locations: partial.y.unwrap_parsed(),
                z_locations: partial.z.unwrap_parsed(),
            };
            Ok((
                Self {
                    rho: rho.unwrap_parsed(),
                },
                locations,
            ))
        }
    }

    impl Add for SpanData {
        type Output = Self;

        fn add(mut self, other: Self) -> Self {
            self.rho
                .iter_mut()
                .zip(other.rho.into_iter())
                .for_each(|(s, o)| *s = *s + o);
            self
        }
    }

    impl Div<f64> for SpanData {
        type Output = Self;

        fn div(mut self, other: f64) -> Self::Output {
            self.rho.iter_mut().for_each(|s| *s = *s / other);
            self
        }
    }

    impl Sub for SpanData {
        type Output = Self;

        fn sub(mut self, other: Self) -> Self {
            self.rho
                .iter_mut()
                .zip(other.rho.into_iter())
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
            let rho = x.into_iter().map(|x| x.data.data.rho).flatten().collect();
            SpanData { rho }
        }
    }

    struct AppendedParse {
        rho: Vec<f64>,
        u: Vec<f64>,
        v: Vec<f64>,
        w: Vec<f64>,
    }

    impl crate::traits::ParseDataArray for AppendedParse {
        fn parse_dataarrays(
            data: &[u8],
            span_info: &super::LocationSpans,
            locations: super::xml_parse::LocationsPartial,
        ) -> Result<(Self, super::Locations), super::xml_parse::ParseError> {
            let mut binary_info: Vec<&mut crate::parse::OffsetBuffer> = Vec::new();
            //
            let len = span_info.x_len() * span_info.y_len() * span_info.z_len();
            let (data, rho) = crate::parse::parse_dataarray_or_lazy(data, b"rho", len)?;
            let (data, u) = crate::parse::parse_dataarray_or_lazy(data, b"u", len)?;
            let (data, v) = crate::parse::parse_dataarray_or_lazy(data, b"v", len)?;
            let (data, w) = crate::parse::parse_dataarray_or_lazy(data, b"w", len)?;

            let mut locations_x__ = crate::parse::PartialDataArrayBuffered::new(locations.x, len);
            let mut locations_y__ = crate::parse::PartialDataArrayBuffered::new(locations.y, len);
            let mut locations_z__ = crate::parse::PartialDataArrayBuffered::new(locations.z, len);

            let mut rho = crate::parse::PartialDataArrayBuffered::new(rho, len);
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

            match &mut rho {
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

            let rho = rho.into_buffer();
            let u = u.into_buffer();
            let v = v.into_buffer();
            let w = w.into_buffer();

            Ok((Self { rho, u, v, w }, locations))
        }
    }
}
