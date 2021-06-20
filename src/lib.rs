mod combine_vtk;
mod data;
mod iter;
pub mod traits;
mod write_vtk;
pub mod xml_parse;

pub(crate) use traits::{DataArray, ParseDataArray};

pub use combine_vtk::combine_vtk;
pub use data::{LocationSpans, Locations, VtkData};
pub use write_vtk::write_dataarray;
pub use write_vtk::write_vtk;
pub use xml_parse::read_and_parse as read_vtk;
pub use xml_parse::NomErrorOwned;

pub use xml::EventWriter;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("An io error occured: `{0}`")]
    IoError(#[from] std::io::Error),
    #[error("The xml data inputted was malformed: `{0}`")]
    XmlError(#[from] xml::reader::Error),
    #[error("Error when parsing the xml data: `{0}`")]
    NomError(#[from] xml_parse::NomErrorOwned),
    #[error("Could not convert file to uf8 encoding: `{0}`")]
    Utf8Error(#[from] std::string::FromUtf8Error),
    #[error("Could not write XML data to file: `{0}`")]
    XmlWriteError(#[from] xml::writer::Error),
}
