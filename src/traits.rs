//! # Traits
//!
//! These are general purpose traits that are required to work with reading, writing, and
//! combining vtk files. With the `derive` feature, the two most important traits
//! `ParseDataArray` and `DataArray` can be derived for you automatically. There are
//! some limitations to this, be sure to refer to each trait's documentation.
//!

use crate::parse;
use crate::Error;
use crate::ParseError;
use nom::IResult;
use std::cell::RefMut;
use std::io::Write;
use xml::writer::EventWriter;

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
pub trait DataArray<Encoding> {
    /// Write all the arrays in the <PointData> section of the file
    ///
    /// If the encoding is base64 or ascii, this function should write the data in the element.
    /// If the encoding is binary, then this function will only write information about the length
    /// and offset of the arrays and `write_mesh_appended` will handle writing the binary data.
    fn write_array_header<W: Write>(
        &self,
        writer: &mut EventWriter<W>,
        starting_offset: i64,
    ) -> Result<(), crate::Error>;

    /// If the encoding is binary, write all of the binary information to the appended
    /// section of the binary file (raw bytes)
    fn write_array_appended<W: Write>(
        &self,
        writer: &mut EventWriter<W>,
    ) -> Result<(), crate::Error>;
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
pub trait Temp {}

pub trait Array {
    fn write_ascii<W: Write>(
        &self,
        writer: &mut EventWriter<W>,
        name: &str,
    ) -> Result<(), crate::Error>;

    fn write_base64<W: Write>(
        &self,
        writer: &mut EventWriter<W>,
        name: &str,
    ) -> Result<(), crate::Error>;

    /// write the file data to the file to the appended section in binary form
    ///
    /// You must ensure that you have called `write_appended_dataarray_header` with
    /// the correct offset before calling this function.
    fn write_binary<W: Write>(&self, writer: &mut EventWriter<W>) -> Result<(), crate::Error>;

    fn length(&self) -> usize;

    fn components(&self) -> usize {
        1
    }
}

pub trait FromBuffer<SPAN> {
    fn from_buffer(buffer: Vec<f64>, spans: &SPAN, components: usize) -> Self;
}

impl <T>FromBuffer<T> for Vec<f64> {
    fn from_buffer(
        buffer: Vec<f64>,
        _spans: &T,
        _components: usize,
    ) -> Self {
        buffer
    }
}

impl FromBuffer<crate::Spans3D> for ndarray::Array4<f64> {
    fn from_buffer(buffer: Vec<f64>, spans: &crate::Spans3D, components: usize) -> Self {
        let mut arr = Self::from_shape_vec((spans.x_len(), spans.y_len(), spans.z_len(), components), buffer).unwrap();
        // this axes swap accounts for how the data is read. It shoud now match _exactly_
        // how the information is input
        arr.swap_axes(0, 2);
        arr
    }
}

pub trait Domain<Encoding> {
    /// Write the mesh information within the `<Coordinates>` section of the file
    ///
    /// If the encoding is base64 or ascii, this function should write the data in the element.
    /// If the encoding is binary, then this function will only write information about the length
    /// and offset of the arrays and `write_mesh_appended` will handle writing the binary data.
    fn write_mesh_header<W: Write>(&self, writer: &mut EventWriter<W>) -> Result<(), Error>;

    /// If writing binary encoded data, this function writes raw binary information to the writer.
    ///
    /// If the encoding is base64 / ascii, this function does nothing.
    fn write_mesh_appended<W: Write>(&self, writer: &mut EventWriter<W>) -> Result<(), Error>;

    /// The VTK-formatted span / extent string for location spans contained in the mesh
    fn span_string(&self) -> String;

    /// number of raw bytes (not encoded in base64 / ascii) that are contained in this mesh
    fn mesh_bytes(&self) -> usize;
}

pub trait ParseMesh {
    type Visitor;
}

pub trait Visitor<Spans>
where
    Self: Sized,
{
    type Output;

    fn read_headers<'a>(spans: &Spans, buffer: &'a [u8]) -> IResult<&'a [u8], Self>;

    fn add_to_appended_reader<'a, 'b>(
        &'a self,
        buffer: &'b mut Vec<RefMut<'a, parse::OffsetBuffer>>,
    );

    fn finish(self, spans: &Spans) -> Result<Self::Output, ParseError>;
}

pub trait ParseArray {
    type Visitor;
}

pub trait ParseSpan {
    fn from_str(extent: &str) -> Self;
}

pub trait Encode {
    fn is_binary() -> bool;
}


#[cfg(feature = "derive")]
mod testgen {
    //use vtk::prelude::*;
    use crate as vtk;

    #[derive(vtk::DataArray, vtk::ParseArray)]
    #[vtk_parse(spans="vtk::Spans3D")]
    #[vtk_write(encoding="binary")]
    pub struct Info {
        a: Vec<f64>,
    }
}
