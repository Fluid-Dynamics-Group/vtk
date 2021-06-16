mod combine_vtk;
mod data;
mod iter;
mod traits;
mod write_vtk;
mod xml_parse;

pub use combine_vtk::combine_vtk;
pub use data::{LocationSpans, Locations, SpanData, VtkData};
pub use data::{LocationsBuilder, SpanDataBuilder};
pub use iter::PointData;
pub use traits::{Data, DataArray};
pub use write_vtk::vtk_to_file as write_vtk;
/// Write a vector of elements to a xml document
pub use write_vtk::write_dataarray;
pub use xml_parse::read_and_parse as read_vtk;
pub use xml_parse::NomErrorOwned;
