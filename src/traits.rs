use crate::Error;
use std::io::Write;
use xml::EventWriter;

pub trait DataArray {
    fn write_dataarray<W: Write>(self, writer: &mut EventWriter<W>) -> Result<(), Error>;
}

pub trait Data: std::fmt::Debug + Default + Clone + PartialEq {}
