use crate::traits::Array;
use std::io::Write;
use xml::writer::{EventWriter, XmlEvent};

impl Array for Vec<f64> {
    fn write_ascii<W: Write>(
        &self,
        writer: &mut EventWriter<W>,
        name: &str,
    ) -> Result<(), crate::Error> {
        self.as_slice().write_ascii(writer, name)
    }
    fn write_base64<W: Write>(
        &self,
        writer: &mut EventWriter<W>,
        name: &str,
    ) -> Result<(), crate::Error> {
        self.as_slice().write_base64(writer, name)
    }
    fn write_binary<W: Write>(&self, writer: &mut EventWriter<W>) -> Result<(), crate::Error> {
        self.as_slice().write_binary(writer)
    }

    fn length(&self) -> usize {
        self.len()
    }
}

impl Array for &[f64] {
    fn write_ascii<W: Write>(
        &self,
        writer: &mut EventWriter<W>,
        name: &str,
    ) -> Result<(), crate::Error> {
        crate::write_vtk::write_inline_array_header(
            writer,
            crate::write_vtk::Encoding::Ascii,
            name,
            1,
        )?;
        let data : String =
            // write out all numbers with 12 points of precision
            self.into_iter()
                .map(|x| {
                    let mut buffer = ryu::Buffer::new();
                    let mut num = buffer.format(*x).to_string();
                    num.push(' ');
                    num
                })
                .collect();

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
            1,
        )?;
        let mut byte_data: Vec<u8> = Vec::with_capacity((self.len() + 1) * 8);

        // for some reason paraview expects the first 8 bytes to be garbage information -
        // I have no idea why this is the case but the first 8 bytes must be ignored
        // for things to work correctly
        byte_data.extend_from_slice("12345678".as_bytes());

        // convert the floats into LE bytes
        self.into_iter()
            .for_each(|float| byte_data.extend_from_slice(&float.to_le_bytes()));

        // encode as base64
        let data = base64::encode(byte_data.as_slice());

        writer.write(XmlEvent::Characters(&data))?;

        crate::write_vtk::close_inline_array_header(writer)?;

        Ok(())
    }

    fn write_binary<W: Write>(&self, writer: &mut EventWriter<W>) -> Result<(), crate::Error> {
        let writer = writer.inner_mut();
        let mut bytes = Vec::with_capacity(self.len() * 8);

        // edge case: if the array ends with 0.0 then any following data arrays will fail to parse
        // see https://gitlab.kitware.com/paraview/paraview/-/issues/20982
        if self[self.len() - 1] == 0.0 {
            // skip the last data point (since we know its 0.0 and
            // instead write a very small number in its place
            self[0..self.len() - 1]
                .into_iter()
                .for_each(|float| bytes.extend(float.to_le_bytes()));

            bytes.extend(0.000001_f64.to_le_bytes());
        } else {
            self.into_iter()
                .for_each(|float| bytes.extend(float.to_le_bytes()));
        }

        writer.write_all(&bytes)?;

        Ok(())
    }

    fn length(&self) -> usize {
        self.len()
    }
}

impl Array for ndarray::Array4<f64> {
    fn write_ascii<W: Write>(
        &self,
        writer: &mut EventWriter<W>,
        name: &str,
    ) -> Result<(), crate::Error> {
        let (nx, ny, nz, components) = self.dim();

        crate::write_vtk::write_inline_array_header(
            writer,
            crate::write_vtk::Encoding::Ascii,
            name,
            components,
        )?;
        let mut data = String::new();

        // convert the x-space array to bytes that can be written to a vtk file
        for k in 0..nz {
            for j in 0..ny {
                for i in 0..nx {
                    for n in 0..components {
                        let float = self.get((i, j, k, n)).unwrap();
                        let mut buffer = ryu::Buffer::new();
                        let mut num = buffer.format(*float).to_string();
                        num.push(' ');
                        data.push_str(&num)
                    }
                }
            }
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
        let (nx, ny, nz, components) = self.dim();

        crate::write_vtk::write_inline_array_header(
            writer,
            crate::write_vtk::Encoding::Base64,
            name,
            components,
        )?;
        let mut byte_data: Vec<u8> = Vec::with_capacity((self.len() + 1) * 8);

        // for some reason paraview expects the first 8 bytes to be garbage information -
        // I have no idea why this is the case but the first 8 bytes must be ignored
        // for things to work correctly
        byte_data.extend_from_slice("12345678".as_bytes());

        // convert the x-space array to bytes that can be written to a vtk file
        for k in 0..nz {
            for j in 0..ny {
                for i in 0..nx {
                    for n in 0..components {
                        let float = self.get((i, j, k, n)).unwrap();
                        byte_data.extend_from_slice(&float.to_le_bytes());
                    }
                }
            }
        }

        // encode as base64
        let data = base64::encode(byte_data.as_slice());

        writer.write(XmlEvent::Characters(&data))?;

        crate::write_vtk::close_inline_array_header(writer)?;

        Ok(())
    }

    fn write_binary<W: Write>(&self, writer: &mut EventWriter<W>) -> Result<(), crate::Error> {
        let writer = writer.inner_mut();
        let mut bytes = Vec::with_capacity(self.len() * 8);

        let (nx, ny, nz, components) = self.dim();

        // convert the x-space array to bytes that can be written to a vtk file
        for k in 0..nz {
            for j in 0..ny {
                for i in 0..nx {
                    for n in 0..components {
                        let float = self.get((i, j, k, n)).unwrap();
                        bytes.extend(float.to_le_bytes());
                    }
                }
            }
        }

        // handle the edge case of the last element in the array being zero
        if *self.get((nx - 1, ny - 1, nz - 1, components - 1)).unwrap() == 0.0 {
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
        self.len()
    }

    fn components(&self) -> usize {
        let (_, _, _, components) = self.dim();
        components
    }
}
