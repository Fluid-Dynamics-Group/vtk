use crate::prelude::*;
use std::marker::PhantomData;

#[derive(Debug, Clone, PartialEq)]
/// Full information on a 2D computational domain. If you are writing
/// a vtk file, this is a candidate type to store in the `domain` field
/// of [VtkData](`crate::VtkData`)
pub struct Rectilinear2D<Encoding> {
    pub spans: Spans2D,
    pub mesh: Mesh2D<Encoding>,
}

impl<Encoding> Rectilinear2D<Encoding> {
    /// create a new domain from mesh information and span information.
    pub fn new(mesh: Mesh2D<Encoding>, spans: Spans2D) -> Rectilinear2D<Encoding> {
        Self { mesh, spans }
    }
}

// from impl is required for generic parsing
impl<T> From<(Mesh2D<T>, Spans2D)> for Rectilinear2D<T> {
    fn from(x: (Mesh2D<T>, Spans2D)) -> Self {
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
pub struct Mesh2D<Encoding> {
    pub x_locations: Vec<f64>,
    pub y_locations: Vec<f64>,
    _marker: PhantomData<Encoding>,
}

impl<Encoding> Mesh2D<Encoding> {
    /// Constructor for the 2D mesh. Encoding can easily
    /// be specified with a turbofish or type inference in later code.
    pub fn new(x_locations: Vec<f64>, y_locations: Vec<f64>) -> Mesh2D<Encoding> {
        Self {
            x_locations,
            y_locations,
            _marker: PhantomData,
        }
    }

    /// swap encodings for this type. This does not change any
    /// of the underlying data
    pub fn change_enconding<T>(self) -> Mesh2D<T> {
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

impl<T, V> PartialEq<Mesh2D<V>> for Mesh2D<T> {
    fn eq(&self, other: &Mesh2D<V>) -> bool {
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
            x_start: 0,
            x_end: nx - 1,
            y_start: 0,
            y_end: ny - 1,
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
            "{} {} {} {} 0 0",
            self.x_start, self.x_end, self.y_start, self.y_end
        )
    }
}

impl ParseSpan for Spans2D {
    fn from_str(extent: &str) -> Self {
        Spans2D::from_span_string(extent)
    }
}

impl Domain<Binary> for Rectilinear2D<Binary> {
    // only write the headers here
    fn write_mesh_header<W: Write>(&self, writer: &mut EventWriter<W>) -> Result<(), Error> {
        let mut offset = 0;

        write_vtk::write_appended_dataarray_header(writer, "X", offset, 1)?;
        offset += (std::mem::size_of::<f64>() * (self.mesh.x_locations.len())) as i64;

        write_vtk::write_appended_dataarray_header(writer, "Y", offset, 1)?;
        offset += (std::mem::size_of::<f64>() * (self.mesh.y_locations.len())) as i64;

        write_vtk::write_appended_dataarray_header(writer, "Z", offset, 1)?;
        //offset += std::mem::size_of::<f64>() as i64;

        Ok(())
    }

    //
    fn write_mesh_appended<W: Write>(&self, writer: &mut EventWriter<W>) -> Result<(), Error> {
        self.mesh.x_locations.write_binary(writer)?;
        self.mesh.y_locations.write_binary(writer)?;
        Ok(())
    }

    fn span_string(&self) -> String {
        self.spans.to_string()
    }

    fn mesh_bytes(&self) -> usize {
        let mut offset = 0;

        offset += std::mem::size_of::<f64>() * (self.mesh.x_locations.len());
        offset += std::mem::size_of::<f64>() * (self.mesh.y_locations.len());
        offset += std::mem::size_of::<f64>() ;

        offset
    }
}

impl Domain<Ascii> for Rectilinear2D<Ascii> {
    // only write the headers here
    fn write_mesh_header<W: Write>(&self, writer: &mut EventWriter<W>) -> Result<(), Error> {
        self.mesh.x_locations.write_ascii(writer, "X")?;
        self.mesh.y_locations.write_ascii(writer, "Y")?;
        vec![0.].write_ascii(writer, "Z")?;

        Ok(())
    }

    //
    fn write_mesh_appended<W: Write>(&self, _: &mut EventWriter<W>) -> Result<(), Error> {
        Ok(())
    }

    fn span_string(&self) -> String {
        self.spans.to_string()
    }

    fn mesh_bytes(&self) -> usize {
        let mut offset = 0;

        offset += std::mem::size_of::<f64>() * (self.mesh.x_locations.len());
        offset += std::mem::size_of::<f64>() * (self.mesh.y_locations.len());
        offset += std::mem::size_of::<f64>();

        offset
    }
}

impl<T> ParseMesh for Mesh2D<T> {
    type Visitor = Mesh2DVisitor;
}

#[doc(hidden)]
pub struct Mesh2DVisitor {
    x_locations: parse::PartialDataArrayBuffered,
    y_locations: parse::PartialDataArrayBuffered,
}

impl Visitor<Spans2D> for Mesh2DVisitor {
    type Output = Mesh2D<Binary>;

    fn read_headers<'a>(spans: &Spans2D, buffer: &'a [u8]) -> IResult<&'a [u8], Self> {
        let (rest, x) = parse::parse_dataarray_or_lazy(buffer, b"X", spans.x_len())?;
        let (rest, y) = parse::parse_dataarray_or_lazy(rest, b"Y", spans.y_len())?;
        let (rest, _) = parse::parse_dataarray_or_lazy(rest, b"Z", spans.y_len())?;

        let x_locations = parse::PartialDataArrayBuffered::new(x, spans.x_len());
        let y_locations = parse::PartialDataArrayBuffered::new(y, spans.y_len());

        let visitor = Self {
            x_locations,
            y_locations,
        };

        Ok((rest, visitor))
    }

    fn add_to_appended_reader<'a, 'b>(
        &'a self,
        buffer: &'b mut Vec<RefMut<'a, parse::OffsetBuffer>>,
    ) {
        self.x_locations.append_to_reader_list(buffer);
        self.y_locations.append_to_reader_list(buffer);
    }

    fn finish(self, _spans: &Spans2D) -> Result<Self::Output, ParseError> {
        let x_locations = self.x_locations.into_buffer();
        let y_locations = self.y_locations.into_buffer();

        Ok(Mesh2D::new(x_locations, y_locations))
    }
}
