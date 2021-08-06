#[cfg(feature = "derive")]
use crate as vtk;

use std::io::Write;
use xml::EventWriter;

/// describes how to write the data to a vtk file
pub trait DataArray {
    fn write_inline_dataarrays<W: Write>(
        &self,
        #[allow(unused_variables)] writer: &mut EventWriter<W>,
    ) -> Result<(), crate::Error> {
        Ok(())
    }
    fn is_appended_array() -> bool {
        true
    }
    fn write_appended_dataarray_headers<W: Write>(
        &self,
        writer: &mut EventWriter<W>,
        starting_offset: i64
    ) -> Result<(), crate::Error>;
    fn write_appended_dataarrays<W: Write>(
        &self,
        writer: &mut EventWriter<W>,
    ) -> Result<(), crate::Error>;
}

pub trait Extender {
    type Extender;

    fn extend_all(self, extender: &mut Self::Extender);
}

/// helper trait to work with an iterator over a vtk
pub trait PointData {
    /// if Data contains a field of Vec<T>, this is just the T
    type PointData;
    fn get_point_data(&self, idx: usize) -> Option<Self::PointData>;
}

/// Descibes how the combining of a set of vtk files should be done
pub trait Combine {
    /// the total number of mpi processes used to generate the data
    fn total_procs(&self) -> usize;
    /// (x start location, x end location)
    fn x_dims(&self) -> (usize, usize);
    /// (y start location, y end location)
    fn y_dims(&self) -> (usize, usize);
    /// (z start location, z end location)
    fn z_dims(&self) -> (usize, usize);
    /// a vector of all the x points in space at which we have some data to write
    fn x_locations(&self) -> Vec<f64>;
    /// a vector of all the y points in space at which we have some data to write
    fn y_locations(&self) -> Vec<f64>;
    /// a vector of all the z points in space at which we have some data to write
    fn z_locations(&self) -> Vec<f64>;
}

/// Describes how to read in a vtk file's data
pub trait ParseDataArray {
    fn parse_dataarrays(
        data: &str,
        span_info: &super::LocationSpans,
    ) -> Result<Self, super::xml_parse::NomErrorOwned>
    where
        Self: Sized;
}

#[cfg(feature = "derive")]
#[derive(vtk_derive::DataArray)]
struct Info<'a> {
    a: Vec<f64>,
    b: &'a [f64],
}

#[cfg(feature = "derive")]
#[derive(vtk_derive::ParseDataArray, vtk_derive::DataArray)]
struct Parse {
    #[allow(dead_code)]
    a: Vec<f64>,
    #[allow(dead_code)]
    b: Vec<f64>,
    #[allow(dead_code)]
    c: Vec<f64>,
}
