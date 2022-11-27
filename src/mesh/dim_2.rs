use crate::prelude::*;

use std::io::BufRead;
use std::marker::PhantomData;

#[derive(Debug, Clone, PartialEq)]
/// Full information on a 2D computational domain. If you are writing
/// a vtk file, this is a candidate type to store in the `domain` field
/// of [VtkData](`crate::VtkData`)
pub struct Rectilinear2D<NUM, Encoding> {
    pub spans: Spans2D,
    pub mesh: Mesh2D<NUM, Encoding>,
}

impl<NUM, Encoding> Rectilinear2D<NUM, Encoding> {
    /// create a new domain from mesh information and span information.
    pub fn new(mesh: Mesh2D<NUM, Encoding>, spans: Spans2D) -> Rectilinear2D<NUM, Encoding> {
        Self { mesh, spans }
    }
}

// from impl is required for generic parsing
impl<NUM, T> From<(Mesh2D<NUM, T>, Spans2D)> for Rectilinear2D<NUM, T> {
    fn from(x: (Mesh2D<NUM, T>, Spans2D)) -> Self {
        Self::new(x.0, x.1)
    }
}

/// Describes the computational stencil for 2D rectilinear geometry
///
/// ## Encoding Type
///
/// This type carries type level information on what kind of encoding to use with the mesh.
/// While this is not explicitly required for any `impl` or trait, it is useful to prevent
/// the end user from having to specify several generic types when using
/// [write_vtk](`crate::write_vtk()`).
///
#[derive(Debug, Clone)]
pub struct Mesh2D<NUM, Encoding> {
    pub x_locations: Vec<NUM>,
    pub y_locations: Vec<NUM>,
    _marker: PhantomData<Encoding>,
}

impl<NUM, Encoding> Mesh2D<NUM, Encoding> {
    /// Constructor for the 2D mesh. Encoding can easily
    /// be specified with a turbofish or type inference in later code.
    pub fn new(x_locations: Vec<NUM>, y_locations: Vec<NUM>) -> Mesh2D<NUM, Encoding> {
        Self {
            x_locations,
            y_locations,
            _marker: PhantomData,
        }
    }

    /// swap encodings for this type. This does not change any
    /// of the underlying data
    pub fn change_encoding<T>(self) -> Mesh2D<NUM, T> {
        let Mesh2D {
            x_locations,
            y_locations,
            _marker,
        } = self;

        Mesh2D {
            x_locations,
            y_locations,
            _marker: PhantomData::<T>,
        }
    }
}

impl<T, V, NUM> PartialEq<Mesh2D<NUM, V>> for Mesh2D<NUM, T>
where
    NUM: PartialEq,
{
    fn eq(&self, other: &Mesh2D<NUM, V>) -> bool {
        self.x_locations == other.x_locations && self.y_locations == other.y_locations
    }
}

/// Describes the area of the computational
/// domain that this VTK handles.
///
/// Most often you want to use the [`Spans2D::new`] constructor
/// if you are not writing multiple vtk files to describe
/// parts of the same domain
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Spans2D {
    pub x_start: usize,
    pub x_end: usize,
    pub y_start: usize,
    pub y_end: usize,
}

impl Spans2D {
    /// create a simple span geometry from some known point lengths
    pub fn new(nx: usize, ny: usize) -> Self {
        Self {
            x_start: 1,
            x_end: nx,
            y_start: 1,
            y_end: ny,
        }
    }

    /// simple constructor used to generate a `LocationSpans` from a string
    /// you would find in a vtk file. The expeceted input is in the form
    /// `"x_start x_end y_start y_end z_start z_end"`
    ///
    /// # Example
    /// ```
    /// vtk::Spans2D::from_span_string("0 10 0 20");
    /// ```
    ///
    /// ## Panics
    ///
    /// This function panics if there are not 6 `usize` values
    /// separated by a single space each
    pub fn from_span_string(span_string: &str) -> Self {
        let mut split = span_string.split_ascii_whitespace();

        Spans2D {
            x_start: split.next().unwrap().parse().unwrap(),
            x_end: split.next().unwrap().parse().unwrap(),
            y_start: split.next().unwrap().parse().unwrap(),
            y_end: split.next().unwrap().parse().unwrap(),
        }
    }

    /// Get the total length in the X direction for this
    /// local segment as paraview would interpret it
    pub fn x_len(&self) -> usize {
        self.x_end - self.x_start + 1
    }

    /// Get the total length in the Y direction for this
    /// local segment as paraview would interpret it
    pub fn y_len(&self) -> usize {
        self.y_end - self.y_start + 1
    }

    /// Format the spans into a string that would be written to a vtk file
    pub(crate) fn to_string(&self) -> String {
        format!(
            "{} {} {} {} 1 1",
            self.x_start, self.x_end, self.y_start, self.y_end
        )
    }
}

impl ParseSpan for Spans2D {
    fn from_str(extent: &str) -> Self {
        Spans2D::from_span_string(extent)
    }
}

impl Span for Spans2D {
    fn num_elements(&self) -> usize {
        self.x_len() * self.y_len() * 1
    }
}

impl<NUM> Domain<Binary> for Rectilinear2D<NUM, Binary>
where
    NUM: Numeric,
{
    // only write the headers here
    fn write_mesh_header<W: Write>(&self, writer: &mut Writer<W>) -> Result<(), Error> {
        let mut offset = 0;

        write_vtk::write_appended_dataarray_header(writer, "X", offset, 1, NUM::as_precision())?;
        offset += (std::mem::size_of::<NUM>() * (self.mesh.x_locations.len())) as i64;

        write_vtk::write_appended_dataarray_header(writer, "Y", offset, 1, NUM::as_precision())?;
        offset += (std::mem::size_of::<NUM>() * (self.mesh.y_locations.len())) as i64;

        write_vtk::write_appended_dataarray_header(writer, "Z", offset, 1, NUM::as_precision())?;
        //offset += std::mem::size_of::<NUM>() as i64;

        Ok(())
    }

    //
    fn write_mesh_appended<W: Write>(&self, writer: &mut Writer<W>) -> Result<(), Error> {
        self.mesh.x_locations.write_binary(writer, false)?;
        self.mesh.y_locations.write_binary(writer, false)?;
        vec![NUM::ZERO].write_binary(writer, false)?;
        Ok(())
    }

    fn span_string(&self) -> String {
        self.spans.to_string()
    }

    fn mesh_bytes(&self) -> usize {
        let mut offset = 0;

        offset += std::mem::size_of::<NUM>() * (self.mesh.x_locations.len());
        offset += std::mem::size_of::<NUM>() * (self.mesh.y_locations.len());
        offset += std::mem::size_of::<NUM>();

        offset
    }
}

impl<NUM> Domain<Ascii> for Rectilinear2D<NUM, Ascii>
where
    NUM: Numeric,
{
    // only write the headers here
    fn write_mesh_header<W: Write>(&self, writer: &mut Writer<W>) -> Result<(), Error> {
        self.mesh.x_locations.write_ascii(writer, "X")?;
        self.mesh.y_locations.write_ascii(writer, "Y")?;
        vec![NUM::ZERO].write_ascii(writer, "Z")?;

        Ok(())
    }

    //
    fn write_mesh_appended<W: Write>(&self, _: &mut Writer<W>) -> Result<(), Error> {
        Ok(())
    }

    fn span_string(&self) -> String {
        self.spans.to_string()
    }

    fn mesh_bytes(&self) -> usize {
        let mut offset = 0;

        offset += std::mem::size_of::<NUM>() * (self.mesh.x_locations.len());
        offset += std::mem::size_of::<NUM>() * (self.mesh.y_locations.len());
        offset += std::mem::size_of::<NUM>();

        offset
    }
}

impl<T, NUM> ParseMesh for Mesh2D<NUM, T> {
    type Visitor = Mesh2DVisitor<NUM>;
}

#[doc(hidden)]
pub struct Mesh2DVisitor<NUM> {
    x_locations: parse::PartialDataArrayBuffered<NUM>,
    y_locations: parse::PartialDataArrayBuffered<NUM>,
    z_locations: parse::PartialDataArrayBuffered<NUM>,
}

impl<NUM> Visitor<Spans2D> for Mesh2DVisitor<NUM>
where
    NUM: Numeric,
    <NUM as std::str::FromStr>::Err: std::fmt::Debug,
{
    type Output = Mesh2D<NUM, Binary>;
    type Num = NUM;

    fn read_headers<R: BufRead>(
        spans: &Spans2D,
        reader: &mut Reader<R>,
        buffer: &mut Vec<u8>,
    ) -> Result<Self, crate::parse::Mesh> {
        let prec = <NUM as Numeric>::as_precision();

        let x = parse::parse_dataarray_or_lazy(reader, buffer, "X", spans.x_len(), prec)?;
        let y = parse::parse_dataarray_or_lazy(reader, buffer, "Y", spans.y_len(), prec)?;
        let z = parse::parse_dataarray_or_lazy(reader, buffer, "Z", 1, prec)?;

        let x_locations = parse::PartialDataArrayBuffered::new(x, spans.x_len());
        let y_locations = parse::PartialDataArrayBuffered::new(y, spans.y_len());
        let z_locations = parse::PartialDataArrayBuffered::new(z, 1);

        let visitor = Self {
            x_locations,
            y_locations,
            z_locations,
        };

        Ok(visitor)
    }

    fn add_to_appended_reader<'a, 'b>(
        &'a self,
        buffer: &'b mut Vec<RefMut<'a, parse::OffsetBuffer<Self::Num>>>,
    ) {
        self.x_locations.append_to_reader_list(buffer);
        self.y_locations.append_to_reader_list(buffer);
        self.z_locations.append_to_reader_list(buffer);
    }

    fn finish(self, _spans: &Spans2D) -> Self::Output {
        let x_locations = self.x_locations.into_buffer();
        let y_locations = self.y_locations.into_buffer();
        //let z_locations = self.z_locations.into_buffer();

        Mesh2D::new(x_locations, y_locations)
    }
}
