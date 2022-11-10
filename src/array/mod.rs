//! container types for data to be read / written from files

mod scalar_2d;
mod scalar_3d;

mod vector;
mod vector_2d;
mod vector_3d;

use crate::prelude::*;
use crate::traits::Array;
use crate::traits::FromBuffer;
use crate::traits::Numeric;
use std::io::Write;
use xml::writer::{EventWriter, XmlEvent};

pub use scalar_2d::Scalar2D;
pub use scalar_3d::Scalar3D;
pub use vector_2d::Vector2D;
pub use vector_3d::Vector3D;

pub use vector_2d::Vector2DIter;
pub use vector_3d::Vector3DIter;

pub trait Components {
    type Iter;

    fn array_components(&self) -> usize;

    fn length(&self) -> usize;

    // TODO: this trait can be done better with GAT
    // since we can use references
    fn iter(&self) -> Self::Iter;
}

impl<T> FromBuffer<T> for Vec<f64> {
    fn from_buffer(buffer: Vec<f64>, _spans: &T, _components: usize) -> Self {
        buffer
    }
}

impl FromBuffer<crate::Spans3D> for ndarray::Array4<f64> {
    fn from_buffer(buffer: Vec<f64>, spans: &crate::Spans3D, components: usize) -> Self {
        let mut arr = Self::from_shape_vec(
            (spans.x_len(), spans.y_len(), spans.z_len(), components),
            buffer,
        )
        .unwrap();
        // this axes swap accounts for how the data is read. It shoud now match _exactly_
        // how the information is input
        arr.swap_axes(0, 2);
        arr
    }
}

impl<T, NUM> Array for T
where
    T: Components,
    <T as Components>::Iter: Iterator<Item = NUM>,
    NUM: Numeric,
{
    fn write_ascii<W: Write>(
        &self,
        writer: &mut EventWriter<W>,
        name: &str,
    ) -> Result<(), crate::Error> {
        crate::write_vtk::write_inline_array_header(
            writer,
            crate::write_vtk::Encoding::Ascii,
            name,
            self.array_components(),
            NUM::as_precision(),
        )?;

        let mut data = String::new();
        let iter = self.iter();

        for float in iter {
            let mut buffer = ryu::Buffer::new();
            let mut num = buffer.format(float).to_string();
            num.push(' ');
            data.push_str(&num)
        }

        writer.write(XmlEvent::Characters(&data))?;

        crate::write_vtk::close_inline_array_header(writer)?;

        Ok(())
    }

    fn write_base64<W: Write>(
        &self,
        writer: &mut EventWriter<W>,
        name: &str,
    ) -> Result<(), crate::Error> {
        crate::write_vtk::write_inline_array_header(
            writer,
            crate::write_vtk::Encoding::Base64,
            name,
            self.array_components(),
            NUM::as_precision(),
        )?;

        let mut byte_data: Vec<u8> = Vec::with_capacity((self.length() + 1) * 8);

        // for some reason paraview expects the first 8 bytes to be garbage information -
        // I have no idea why this is the case but the first 8 bytes must be ignored
        // for things to work correctly
        byte_data.extend_from_slice("12345678".as_bytes());

        let iter = self.iter();

        for float in iter {
            float.extend_le_bytes(&mut byte_data);
        }

        // encode as base64
        let data = base64::encode(byte_data.as_slice());

        writer.write(XmlEvent::Characters(&data))?;

        crate::write_vtk::close_inline_array_header(writer)?;

        Ok(())
    }

    fn write_binary<W: Write>(
        &self,
        writer: &mut EventWriter<W>,
        is_last: bool,
    ) -> Result<(), crate::Error> {
        let writer = writer.inner_mut();

        let mut iter = self.iter().peekable();

        loop {
            if let Some(float) = iter.next() {
                // edge case: if the array ends with 0.0 then any following data arrays will fail to parse
                // see https://gitlab.kitware.com/paraview/paraview/-/issues/20982
                if !is_last && iter.peek().is_none() && float == NUM::ZERO {
                    NUM::SMALL.write_le_bytes(writer)?;
                } else {
                    float.write_le_bytes(writer)?;
                }
            } else {
                break;
            }
        }

        Ok(())
    }

    fn length(&self) -> usize {
        Components::length(self)
    }

    fn components(&self) -> usize {
        Components::array_components(self)
    }

    fn precision(&self) -> Precision {
        NUM::as_precision()
    }

    fn size_of_elem(&self) -> usize {
        NUM::SIZE
    }
}
