//! # Traits
//!
//! These are general purpose traits that are required to work with reading, writing, and
//! combining vtk files. With the `derive` feature, the two most important traits
//! `ParseDataArray` and `DataArray` can be derived for you automatically. There are
//! some limitations to this, be sure to refer to each trait's documentation.
//!
#[cfg(feature = "derive")]
use crate as vtk;

use std::io::Write;
use xml::EventWriter;

/// describes how to write the data to a vtk file
///
/// There are two main ways to write data to a vtk file. Either you can write the data inline
/// within the `DataArray` attribute or you can write the data as binary to an appended section
/// with a specified offset. Writing the data inline, while more clear, requires either an ascii or
/// base64 encoding which uses significantly more space than the appended data.
///
/// If you want to write the data inline (base64 / ascii), you need to implement the
/// `write_inline_dataarrays` and `is_appended_dataarray_headers` functions:
///
/// ```ignore
/// struct FlowData {
///     u: Vec<f64>,
///     v: Vec<f64>,
///     w: Vec<f64>,
/// }
///
/// impl vtk::traits::DataArray for FlowData {
///     fn write_dataarray<W: Write>( &self, writer: &mut EventWriter<W>) -> Result<(), vtk::Error> {
///         vtk::write_inline_dataarray(writer, &self.u, "u", vtk::Encoding::Base64)?;
///         vtk::write_inline_dataarray(writer, &self.v, "v", vtk::Encoding::Base64)?;
///         vtk::write_inline_dataarray(writer, &self.w, "w", vtk::Encoding::Base64)?;
///         Ok(())
///     }
///
///     fn is_appended_array() -> bool {
///         false
///     }
///
///     // just return anything from these functions, they will not be called
///     fn write_appended_dataarray_headers<W: Write>(
///         &self,
///         writer: &mut EventWriter<W>,
///         starting_offset: i64,
///     ) -> Result<(), crate::Error> {
///         Ok(())
///     }
///
///     // just return anything from these functions, they will not be called
///     fn write_appended_dataarrays<W: Write>(
///         &self,
///         writer: &mut EventWriter<W>,
///     ) -> Result<(), crate::Error> {
///         Ok(())
///     }
/// }
/// ```
///
/// The recommended way of using this trait is deriving. You can encoding into `"binary"`
/// (default), `"ascii"`, or `"base64"`:
///
/// ```ignore
/// // uncommend different encodings to see output file sizes
/// #[derive(vtk::DataArray)]
/// // #[vtk(encoding = "binary") // enabled by default
/// // #[vtk(encoding = "base64")
/// // #[vtk(encoding = "ascii")
/// struct FlowData {
///     u: Vec<f64>,
///     v: Vec<f64>,
///     w: Vec<f64>,
/// }
/// ```
///
/// a VTK file will be automatically generated with the following format (the default binary):
///
/// ```ignore
/// <?xml version="1.0" encoding="UTF-8"?>
/// <VTKFile type="RectilinearGrid" version="1.0" byte_order="LittleEndian" header_type="UInt64">
///     <RectilinearGrid WholeExtent="0 63 0 63 0 63">
///         <Piece Extent="0 63 0 63 0 63">
///             <Coordinates>
///                 <DataArray type="Float64" NumberOfComponents="1" Name="X" format="appended" offset="-8" />
///                 <DataArray type="Float64" NumberOfComponents="1" Name="Y" format="appended" offset="504" />
///                 <DataArray type="Float64" NumberOfComponents="1" Name="Z" format="appended" offset="1016" />
///             </Coordinates>
///             <PointData>
///                 <DataArray type="Float64" NumberOfComponents="1" Name="u" format="appended" offset="1528" />
///                 <DataArray type="Float64" NumberOfComponents="1" Name="v" format="appended" offset="2098680" />
///                 <DataArray type="Float64" NumberOfComponents="1" Name="w" format="appended" offset="4195832" />
///             </PointData>
///         </Piece>
///     </RectilinearGrid>
///     <AppendedData encoding="raw">
///         _binary data here
///     </AppendedData>
/// </VTKFile>
pub trait DataArray {
    fn write_inline_dataarrays<W: Write>(
        &self,
        #[allow(unused_variables)] writer: &mut EventWriter<W>,
    ) -> Result<(), crate::Error> {
        Ok(())
    }
    fn is_appended_array() -> bool {
        false 
    }
    fn write_appended_dataarray_headers<W: Write>(
        &self,
        writer: &mut EventWriter<W>,
        starting_offset: i64,
    ) -> Result<(), crate::Error>;
    fn write_appended_dataarrays<W: Write>(
        &self,
        writer: &mut EventWriter<W>,
    ) -> Result<(), crate::Error>;
}

/// helper trait to work with an iterator over a vtk
///
/// Defines a way to get the data for a single point in a flowfield
/// by a linear index
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
///
/// The built-in routines for parsing DataArrays only account for ascii data stored
/// in an inline element. If your data is base64 encoded or appended as binary in the final
/// section then this parsing will not work for you.
///
/// If you are planning on skipping some of the data in the vtk (not parsing it), then you
/// must ensure that there is no data associated with the `AppendedData` element in the vtk.
/// If some fields are skipped, then the final variable read from `AppendedData` will over-run
/// into data not intended to be parsed into that field. This behavior can 
/// be modified by implementing the trait manually.
///
/// This trait can be derived with the `vtk::ParseDataArray` proc macro:
///
/// ```ignore
/// #[derive(vtk::ParseDataArray)]
/// struct FlowData {
///     u: Vec<f64>,
///     v: Vec<f64>,
///     w: Vec<f64>,
/// }
/// ```
///
/// will automatically parse a vtk file in the following format
///
/// ```ignore
/// <?xml version="1.0" encoding="UTF-8"?>
/// <VTKFile type="RectilinearGrid" version="1.0" byte_order="LittleEndian" header_type="UInt64">
///     <RectilinearGrid WholeExtent="0 63 0 63 0 63">
///         <Piece Extent="0 63 0 63 0 63">
///             <Coordinates>
///                 <DataArray type="Float64" NumberOfComponents="1" Name="X" format="appended" offset="-8" />
///                 <DataArray type="Float64" NumberOfComponents="1" Name="Y" format="appended" offset="504" />
///                 <DataArray type="Float64" NumberOfComponents="1" Name="Z" format="appended" offset="1016" />
///             </Coordinates>
///             <PointData>
///                 <DataArray type="Float64" NumberOfComponents="1" Name="u" format="appended" offset="1528" />
///                 <DataArray type="Float64" NumberOfComponents="1" Name="v" format="appended" offset="2098680" />
///                 <DataArray type="Float64" NumberOfComponents="1" Name="w" format="appended" offset="4195832" />
///             </PointData>
///         </Piece>
///     </RectilinearGrid>
/// </VTKFile>
/// ```
pub trait ParseDataArray {
    fn parse_dataarrays(
        data: &[u8],
        span_info: &super::LocationSpans,
        locations: super::parse::LocationsPartial,
    ) -> Result<(Self, super::Locations), super::parse::ParseError>
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
//#[derive(vtk_derive::DataArray)]
struct Parse {
    #[allow(dead_code)]
    a: Vec<f64>,
    #[allow(dead_code)]
    b: Vec<f64>,
    #[allow(dead_code)]
    c: Vec<f64>,
}
