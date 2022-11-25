#![doc = include_str!("../README.md")]

pub mod array;
mod data;
pub mod mesh;
pub mod parse;
pub mod prelude;
mod traits;
mod utils;
mod write_vtk;

pub use traits::DataArray;
pub use traits::Domain;
pub use traits::ParseArray;
pub use traits::ParseMesh;
pub use traits::Visitor;

pub use data::VtkData;

pub use mesh::{Mesh2D, Rectilinear2D, Spans2D};
pub use mesh::{Mesh3D, Rectilinear3D, Spans3D};

pub use array::{Scalar2D, Scalar3D, Vector2D, Vector3D};

pub use traits::*;
pub use traits::{Array, FromBuffer};
pub use write_vtk::write_vtk;
pub use write_vtk::{write_appended_dataarray_header, write_inline_dataarray, Encoding};

pub use parse::read_and_parse as read_vtk;
//pub use parse::ParseError;

#[cfg(feature = "derive")]
pub use vtk_derive::{DataArray, ParseArray};

pub use ndarray;
//pub use nom;

pub use quick_xml::writer::Writer;
pub use quick_xml::reader::Reader;

/// general purpose error enumeration for possible causes of failure.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("An io error occured: `{0}`")]
    Io(#[from] std::io::Error),
    #[error("Error while parsing VTK xml")]
    Parse(#[from] parse::ParseError),
    #[error("Could not convert file to uf8 encoding: `{0}`")]
    Utf8(#[from] std::string::FromUtf8Error),
    #[error("Could not write XML data to file: `{0}`")]
    XmlWrite(#[from] quick_xml::Error),
}

/// Binary encoding marker type
#[derive(Debug, Clone, PartialEq)]
pub struct Binary;

/// base64 encoding marker type
#[derive(Debug, Clone)]
pub struct Base64;

/// ascii encoding marker type
#[derive(Debug, Clone, PartialEq)]
pub struct Ascii;

impl traits::Encode for Binary {
    fn is_binary() -> bool {
        true
    }
}

impl traits::Encode for Ascii {
    fn is_binary() -> bool {
        false
    }
}

impl traits::Encode for Base64 {
    fn is_binary() -> bool {
        false
    }
}

#[cfg(test)]
mod helpers {
    use super::write_vtk::Encoding;
    use crate as vtk;
    use crate::prelude::*;
    use crate::prelude::*;
    use crate::Binary;
    use std::ops::{Add, Div, Sub};

    #[derive(Debug, Clone, Default, PartialEq)]
    pub struct SpanData {
        pub u: Vec<f64>,
    }

    pub struct SpanDataVisitor {
        u: vtk::parse::PartialDataArrayBuffered,
    }

    impl vtk::Visitor<vtk::Spans3D> for SpanDataVisitor {
        type Output = SpanData;
        fn read_headers<R: BufRead>(
            spans: &vtk::Spans3D,
            reader: &mut Reader<R>,
            buffer: &mut Vec<u8>,
        ) -> Result<Self, crate::parse::Mesh> {
            let u = vtk::parse::parse_dataarray_or_lazy(reader, buffer, "u", 0)?;
            let u = vtk::parse::PartialDataArrayBuffered::new(u, spans.num_elements());
            let visitor = SpanDataVisitor { u };
            Ok(visitor)
        }
        fn add_to_appended_reader<'a, 'b>(
            &'a self,
            buffer: &'b mut Vec<std::cell::RefMut<'a, vtk::parse::OffsetBuffer>>,
        ) {
            self.u.append_to_reader_list(buffer);
        }
        fn finish(self, spans: &vtk::Spans3D) -> Self::Output {
            let comp = self.u.components();
            let u = self.u.into_buffer();
            let u = vtk::FromBuffer::from_buffer(u, &spans, comp);
            SpanData { u }
        }
    }

    impl vtk::ParseArray for SpanData {
        type Visitor = SpanDataVisitor;
    }

    impl vtk::DataArray<vtk::Binary> for SpanData {
        fn write_array_header<W: std::io::Write>(
            &self,
            writer: &mut vtk::Writer<W>,
            offset: i64,
        ) -> Result<(), vtk::Error> {
            let ref_field = &self.u;
            let comps = vtk::Array::components(ref_field);
            vtk::write_appended_dataarray_header(writer, "u", offset, comps, Precision::Float64)?;
            Ok(())
        }
        fn write_array_appended<W: std::io::Write>(
            &self,
            writer: &mut vtk::Writer<W>,
        ) -> Result<(), vtk::Error> {
            vtk::Array::write_binary(&self.u, writer, true)?;
            Ok(())
        }
    }
}
