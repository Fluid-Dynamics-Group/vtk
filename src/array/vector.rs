use crate::prelude::*;
use quick_xml::events::BytesText;
use quick_xml::events::Event;

impl<NUM> Array for Vec<NUM>
where
    NUM: Numeric,
{
    fn write_ascii<W: Write>(
        &self,
        writer: &mut Writer<W>,
        name: &str,
    ) -> Result<(), crate::Error> {
        self.as_slice().write_ascii(writer, name)
    }
    fn write_base64<W: Write>(
        &self,
        writer: &mut Writer<W>,
        name: &str,
    ) -> Result<(), crate::Error> {
        self.as_slice().write_base64(writer, name)
    }
    fn write_binary<W: Write>(
        &self,
        writer: &mut Writer<W>,
        is_last: bool,
    ) -> Result<(), crate::Error> {
        self.as_slice().write_binary(writer, is_last)
    }

    fn length(&self) -> usize {
        self.len()
    }

    fn components(&self) -> usize {
        1
    }

    fn precision(&self) -> Precision {
        NUM::as_precision()
    }

    fn size_of_elem(&self) -> usize {
        NUM::SIZE
    }
}

impl<NUM> Array for &[NUM]
where
    NUM: Numeric,
{
    fn write_ascii<W: Write>(
        &self,
        writer: &mut Writer<W>,
        name: &str,
    ) -> Result<(), crate::Error> {
        crate::write_vtk::write_inline_array_header(
            writer,
            crate::write_vtk::Encoding::Ascii,
            name,
            1,
            NUM::as_precision(),
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

        let data = Event::Text(BytesText::new(&data));
        writer.write_event(data)?;

        crate::write_vtk::close_inline_array_header(writer)?;

        Ok(())
    }
    fn write_base64<W: Write>(
        &self,
        writer: &mut Writer<W>,
        name: &str,
    ) -> Result<(), crate::Error> {
        crate::write_vtk::write_inline_array_header(
            writer,
            crate::write_vtk::Encoding::Base64,
            name,
            1,
            NUM::as_precision(),
        )?;
        let mut byte_data: Vec<u8> = Vec::with_capacity((self.len() + 1) * 8);

        // for some reason paraview expects the first 8 bytes to be garbage information -
        // I have no idea why this is the case but the first 8 bytes must be ignored
        // for things to work correctly
        byte_data.extend_from_slice("12345678".as_bytes());

        // convert the floats into LE bytes
        self.into_iter()
            .for_each(|float| float.extend_le_bytes(&mut byte_data));

        // encode as base64
        let data = base64::encode(byte_data.as_slice());

        let characters = Event::Text(BytesText::new(&data));
        writer.write_event(characters)?;

        crate::write_vtk::close_inline_array_header(writer)?;

        Ok(())
    }

    fn write_binary<W: Write>(
        &self,
        writer: &mut Writer<W>,
        is_last: bool,
    ) -> Result<(), crate::Error> {
        let writer = writer.inner();

        let mut iter = self.iter().peekable();

        loop {
            if let Some(float) = iter.next() {
                // edge case: if the array ends with 0.0 then any following data arrays will fail to parse
                // see https://gitlab.kitware.com/paraview/paraview/-/issues/20982
                if !is_last && iter.peek().is_none() && *float == NUM::ZERO {
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
        self.len()
    }

    fn components(&self) -> usize {
        1
    }

    fn precision(&self) -> Precision {
        NUM::as_precision()
    }

    fn size_of_elem(&self) -> usize {
        NUM::SIZE
    }
}
