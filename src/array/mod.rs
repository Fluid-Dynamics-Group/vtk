//! container types for data to be read / written from files

mod scalar_2d;
mod scalar_3d;

mod field_2d;
mod field_3d;
mod vector;

use crate::traits::Array;
use std::io::Write;
use xml::writer::{EventWriter, XmlEvent};

pub use field_2d::Field2D;

pub trait Components {
    type Iter;

    fn array_components(&self) -> usize;

    fn length(&self) -> usize;

    // TODO: this trait can be done better with GAT
    // since we can use references
    fn iter(&self) -> Self::Iter;
}



impl <T> Array for T
where T: Components,
      <T as Components>::Iter : Iterator<Item=f64>
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
        )?;

        let mut byte_data: Vec<u8> = Vec::with_capacity((self.length() + 1) * 8);

        // for some reason paraview expects the first 8 bytes to be garbage information -
        // I have no idea why this is the case but the first 8 bytes must be ignored
        // for things to work correctly
        byte_data.extend_from_slice("12345678".as_bytes());

        let iter = self.iter();

        for float in iter {
            byte_data.extend_from_slice(&float.to_le_bytes());
        }

        // encode as base64
        let data = base64::encode(byte_data.as_slice());

        writer.write(XmlEvent::Characters(&data))?;

        crate::write_vtk::close_inline_array_header(writer)?;

        Ok(())
    }

    fn write_binary<W: Write>(&self, writer: &mut EventWriter<W>) -> Result<(), crate::Error> {
        let writer = writer.inner_mut();
        let mut bytes = Vec::with_capacity(self.length() * 8);

        let iter = self.iter();

        let mut last = 0.0;

        for float in iter {
            bytes.extend(float.to_le_bytes());
            last = float;
        }

        // handle the edge case of the last element in the array being zero
        if last == 0.0 {
            let mut index = bytes.len() - 9;
            for i in 0.000001_f64.to_le_bytes() {
                bytes[index] = i;
                index += 1
            }
        }

        writer.write_all(&bytes)?;

        Ok(())
    }

    fn length(&self) -> usize {
        Components::length(self)
    }

    fn components(&self) -> usize {
        Components::array_components(self)
    }
}
