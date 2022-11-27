//! # Traits
//!
//! These are general purpose traits that are required to work with reading, writing, and
//! combining vtk files. With the `derive` feature, the two most important traits
//! `ParseDataArray` and `DataArray` can be derived for you automatically. There are
//! some limitations to this, be sure to refer to each trait's documentation.
//!

use crate::parse;
use crate::Error;
use std::cell::RefMut;
use std::io::BufRead;
use std::io::Write;

use crate::prelude::*;

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
///     fn write_dataarray<W: Write>( &self, writer: &mut Writer<W>) -> Result<(), vtk::Error> {
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
///         writer: &mut Writer<W>,
///         starting_offset: i64,
///     ) -> Result<(), crate::Error> {
///         Ok(())
///     }
///
///     // just return anything from these functions, they will not be called
///     fn write_appended_dataarrays<W: Write>(
///         &self,
///         writer: &mut Writer<W>,
///     ) -> Result<(), vtk::Error> {
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
        writer: &mut Writer<W>,
        starting_offset: i64,
    ) -> Result<(), crate::Error>;

    /// If the encoding is binary, write all of the binary information to the appended
    /// section of the binary file (raw bytes)
    fn write_array_appended<W: Write>(&self, writer: &mut Writer<W>) -> Result<(), crate::Error>;
}

/// Information on how to write data from a given array (as part of a larger collection
/// implementing `DataArray.
///
/// This trait is required to be implemented on any types that are being written to a vtk file.
/// You probably want to use one of the provided implementations in
/// [Scalar3D](crate::Scalar3D) [Scalar2D](crate::Scalar2D) [Vector3D](crate::Vector3D) [Vector2D](crate::Vector2D)
pub trait Array {
    /// outputs the information in the data array to ascii encoded data
    fn write_ascii<W: Write>(&self, writer: &mut Writer<W>, name: &str)
        -> Result<(), crate::Error>;

    /// outputs the information in the data array to base64 encoded data
    fn write_base64<W: Write>(
        &self,
        writer: &mut Writer<W>,
        name: &str,
    ) -> Result<(), crate::Error>;

    /// write the file data to the file to the appended section in binary form
    ///
    /// if this is the last array written to the file, `is_last` should be set to true
    fn write_binary<W: Write>(
        &self,
        writer: &mut Writer<W>,
        is_last: bool,
    ) -> Result<(), crate::Error>;

    // the number of elements in this array
    fn length(&self) -> usize;

    // the number of components at each point. For a scalar field this is 1, for a cartesian vector (such as
    // velocity) is 3.
    fn components(&self) -> usize;

    /// get the precision of the data that is being written
    fn precision(&self) -> Precision;

    fn size_of_elem(&self) -> usize;
}

/// Converts a buffer of bytes (as read from a VTK file) to the correct order
/// for your [`Array`] type
pub trait FromBuffer<SPAN> {
    fn from_buffer(buffer: Vec<f64>, spans: &SPAN, components: usize) -> Self;
}

/// Description on how to write the mesh and span information to a vtk file.
///
/// This trait is required to be implemented on the type in the `domain` field
/// of [VtkData](crate::VtkData).
///
/// This type trait is implemented for the [Rectilinear3D](crate::Rectilinear3D)
/// and [Rectilinear2D](crate::Rectilinear2D) types. You probably want to use one of those
/// instead of creating your own.
///
pub trait Domain<Encoding> {
    /// Write the mesh information within the `<Coordinates>` section of the file
    ///
    /// If the encoding is base64 or ascii, this function should write the data in the element.
    /// If the encoding is binary, then this function will only write information about the length
    /// and offset of the arrays and `write_mesh_appended` will handle writing the binary data.
    fn write_mesh_header<W: Write>(&self, writer: &mut Writer<W>) -> Result<(), Error>;

    /// If writing binary encoded data, this function writes raw binary information to the writer.
    ///
    /// If the encoding is base64 / ascii, this function does nothing.
    fn write_mesh_appended<W: Write>(&self, writer: &mut Writer<W>) -> Result<(), Error>;

    /// The VTK-formatted span / extent string for location spans contained in the mesh
    fn span_string(&self) -> String;

    /// number of raw bytes (not encoded in base64 / ascii) that are contained in this mesh
    fn mesh_bytes(&self) -> usize;
}

/// Helper trait to provide type information on a mesh
///
/// For rectilinear data, you can use either [Mesh3D](crate::Mesh3D) or [Mesh2D](crate::Mesh2D).
pub trait ParseMesh {
    type Visitor;
}

/// Handles the bulk of parsing. This trait should be derived.
///
/// Visitors are a pattern the `vtk` crate uses to track partial data throughout the
/// parsing process in a file. Since data can either be stored in where its metadata is
/// specified (ascii and base64 encoded data) or appended to the end of the file at
/// an ambiguous offset, the `Visitor` trait is implemented on a type (From [ParseMesh::Visitor](ParseMesh)
/// for [ParseArray::Visitor](ParseArray)
/// and `Output`'s the type that `ParseMesh` or `ParseArray` is implemented for.
///
///
/// ## Example
///
/// ```
/// // provides `.num_elements()` method for `Spans3D`
/// use vtk::Span;
///
/// #[derive(Debug, Clone, Default, PartialEq)]
/// pub struct SpanData {
///     pub u: Vec<f64>,
/// }
///
/// pub struct SpanDataVisitor {
///     u: vtk::parse::PartialDataArrayBuffered<f64>,
/// }
///
/// impl vtk::Visitor<vtk::Spans3D> for SpanDataVisitor {
///     type Output = SpanData;
///     type Num = f64;
///
///     fn read_headers<R: std::io::BufRead>(
///         spans: &vtk::Spans3D,
///         reader: &mut vtk::Reader<R>,
///         buffer: &mut Vec<u8>,
///     ) -> Result<Self, vtk::parse::Mesh> {
///         let u = vtk::parse::parse_dataarray_or_lazy(reader, buffer, "u", 0)?;
///         let u = vtk::parse::PartialDataArrayBuffered::new(u, spans.num_elements());
///         let visitor = SpanDataVisitor { u };
///         Ok(visitor)
///     }
///     fn add_to_appended_reader<'a, 'b>(
///         &'a self,
///         buffer: &'b mut Vec<std::cell::RefMut<'a, vtk::parse::OffsetBuffer<Self::Num>>>,
///     ) {
///         self.u.append_to_reader_list(buffer);
///     }
///     fn finish(self, spans: &vtk::Spans3D) -> Self::Output {
///         let comp = self.u.components();
///         let u = self.u.into_buffer();
///         let u = vtk::FromBuffer::from_buffer(u, &spans, comp);
///         SpanData { u }
///     }
/// }
/// ```
///
/// Equivalently, this data can simply be created if the derive feature is enabled:
///
/// ```
/// #[derive(Debug, Clone, Default, PartialEq, vtk::ParseArray)]
/// #[vtk_parse(spans="vtk::Spans3D", precision="f64")]
/// pub struct SpanData {
///     pub u: Vec<f64>,
/// }
/// ```
///
pub trait Visitor<Spans>
where
    Self: Sized,
{
    /// The type that will be output from the visitor once parsing is complete
    type Output;
    type Num;

    /// The implementing type is constructed with the `read_headers` function.
    fn read_headers<R: BufRead>(
        spans: &Spans,
        reader: &mut Reader<R>,
        buffer: &mut Vec<u8>,
    ) -> Result<Self, crate::parse::Mesh>;

    /// all the internal buffers that are stored in the visitor type
    /// are added to a vector here so that they can be sorted and read (in order by offset) from the
    /// appended binary section of the vtk file.
    fn add_to_appended_reader<'a, 'b>(
        &'a self,
        buffer: &'b mut Vec<RefMut<'a, parse::OffsetBuffer<Self::Num>>>,
    );

    /// After all the binary data has been read, the `finish` function finalizes any last-minute
    /// changes before returning the full information of the type we have been parsing towards.
    fn finish(self, spans: &Spans) -> Self::Output;
}

/// Generically gives access to information on a span
pub trait Span {
    fn num_elements(&self) -> usize;
}

/// Helper trait to provide type information on a dataarray
///
/// This trait should almost certainly be derived. See the documentation on the [`Visitor`]
/// trait for information on how the visitor works.
///
/// If you are deriving this type, ensure that all the containers within your struct implement
/// [`FromBuffer`], there are no reference types, your type is public, and you correctly specify the correct `Span`
/// object that will be used in the parsing. For 3D rectilinear parsing of some fluids data
/// you could use this:
///
/// ```
/// #[derive(Debug, Clone, Default, PartialEq, vtk::ParseArray)]
/// // we have 3D data so it makes sense to use `Spans3D` here.
/// #[vtk_parse(spans="vtk::Spans3D", precision = "f64")]
/// pub struct SpanData {
///     pub velocity: vtk::Vector3D<f64>,
///     pub pressure: vtk::Scalar3D<f64>,
///     pub density: vtk::Scalar3D<f64>,
/// }
/// ```
pub trait ParseArray {
    type Visitor;
}

/// Transforms a string from the `Extent` or `WholeExtent` header to numerical information
///
/// ## Example
///
/// ```
/// use vtk::ParseSpan;
///
/// // a 10 x 30 x 10 sized domain
/// let extent = "1 10 1 30 1 10";
/// let parsed_extent = vtk::Spans3D::from_str(extent);
/// assert_eq!(parsed_extent, vtk::Spans3D::new(10,30,10))
/// ```
pub trait ParseSpan {
    /// Takes in the `WholeExtent` or `Extent` attributes from the vtk file
    /// and returns size information on the domain
    fn from_str(extent: &str) -> Self;
}

/// Describes the encoding of a marker type
pub trait Encode {
    fn is_binary() -> bool;
}

#[cfg(feature = "derive")]
mod testgen {
    //use vtk::prelude::*;
    use crate as vtk;

    #[derive(vtk::DataArray, vtk::ParseArray)]
    #[vtk_parse(spans = "vtk::Spans3D", precision = "f64")]
    #[vtk_write(encoding = "binary")]
    pub struct Info {
        a: Vec<f64>,
    }
}

/// A trait to abstract over [`f64`] and [`f32`] container data types
pub trait Numeric: std::cmp::PartialEq<Self> + ryu::Float + Sized + std::str::FromStr {
    const SIZE: usize = std::mem::size_of::<Self>();
    const ZERO: Self;
    const SMALL: Self;

    fn extend_le_bytes(&self, byte_list: &mut Vec<u8>);

    fn write_le_bytes<W: Write>(&self, byte_list: &mut W) -> Result<(), std::io::Error>;

    fn as_precision() -> crate::write_vtk::Precision;

    fn bytes_to_float(bytes: &[u8]) -> Self;
}

impl Numeric for f32 {
    const ZERO: Self = 0.0f32;
    const SMALL: Self = 0.000001f32;

    fn extend_le_bytes(&self, byte_list: &mut Vec<u8>) {
        byte_list.extend(self.to_le_bytes())
    }

    fn write_le_bytes<W: Write>(&self, byte_list: &mut W) -> Result<(), std::io::Error> {
        byte_list.write_all(&self.to_le_bytes())
    }

    fn as_precision() -> crate::write_vtk::Precision {
        crate::write_vtk::Precision::Float32
    }

    fn bytes_to_float(bytes: &[u8]) -> Self {
        let mut arr = [0; 4];
        bytes
            .into_iter()
            .enumerate()
            .for_each(|(idx, value)| arr[idx] = *value);
        f32::from_le_bytes(arr)
    }
}

impl Numeric for f64 {
    const ZERO: Self = 0.0f64;
    const SMALL: Self = 0.000001f64;

    fn extend_le_bytes(&self, byte_list: &mut Vec<u8>) {
        byte_list.extend(self.to_le_bytes())
    }

    fn write_le_bytes<W: Write>(&self, byte_list: &mut W) -> Result<(), std::io::Error> {
        byte_list.write_all(&self.to_le_bytes())
    }

    fn as_precision() -> crate::write_vtk::Precision {
        crate::write_vtk::Precision::Float64
    }

    fn bytes_to_float(bytes: &[u8]) -> Self {
        let mut arr = [0; 8];
        bytes
            .into_iter()
            .enumerate()
            .for_each(|(idx, value)| arr[idx] = *value);
        f64::from_le_bytes(arr)
    }
}
