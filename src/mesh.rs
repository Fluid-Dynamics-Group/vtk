use crate::traits::*;
use crate::write_vtk;
use crate::Binary;
use crate::Error;
use crate::EventWriter;
use crate::Visitor;
use crate::parse;
use crate::ParseError;
use crate::ParseMesh;
use std::io::Write;
use std::marker::PhantomData;
use nom::IResult;
use std::cell::RefCell;
use std::cell::RefMut;

pub struct Rectilinear3D<Encoding> {
    pub spans: Spans3D,
    pub mesh: Mesh3D<Encoding>
}

impl<Encoding> Rectilinear3D<Encoding> {
    pub fn new(
        mesh: Mesh3D<Encoding>,
        spans: Spans3D,
    ) -> Rectilinear3D<Encoding> {

        Self {
            mesh,
            spans,
        }
    }
}

/// Describes the computational stencil for 3D rectilinear geometry
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Mesh3D<Encoding> {
    pub x_locations: Vec<f64>,
    pub y_locations: Vec<f64>,
    pub z_locations: Vec<f64>,
    pub spans: Spans3D,
    _marker: PhantomData<Encoding>,
}

impl<Encoding> Mesh3D <Encoding> {
    pub fn new(
        x_locations: Vec<f64>,
        y_locations: Vec<f64>,
        z_locations: Vec<f64>,
        spans: Spans3D,
    ) -> Mesh3D<Encoding> {
        Self {
            x_locations,
            y_locations,
            z_locations,
            spans,
            _marker: PhantomData
        }
    }
}


/// Describes the area of the computational 
/// domain that this VTK handles. 
///
/// Most often you want to use the [`new`] constructor 
/// if you are not writing multiple vtk files to describe
/// parts of the same domain
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Spans3D {
    pub x_start: usize,
    pub x_end: usize,
    pub y_start: usize,
    pub y_end: usize,
    pub z_start: usize,
    pub z_end: usize,
}

impl Spans3D {

    /// create a simple span geometry from some known point lengths
    pub fn new(nx: usize, ny: usize, nz: usize) -> Self {
        Self {
            x_start: 0,
            x_end: nx-1,
            y_start: 0,
            y_end: ny-1,
            z_start: 0,
            z_end: nz-1,
        }
    }

    /// simple constructor used to generate a `LocationSpans` from a string
    /// you would find in a vtk file. The expeceted input is in the form
    /// `"x_start x_end y_start y_end z_start z_end"`
    ///
    /// # Example
    /// ```
    /// vtk::LocationSpans::new("0 10 0 20 0 10");
    /// ```
    ///
    /// ## Panics
    ///
    /// This function panics if there are not 6 `usize` values
    /// separated by a single space each
    pub fn from_span_string(span_string: &str) -> Self {
        let mut split = span_string.split_ascii_whitespace();

        Spans3D {
            x_start: split.next().unwrap().parse().unwrap(),
            x_end: split.next().unwrap().parse().unwrap(),
            y_start: split.next().unwrap().parse().unwrap(),
            y_end: split.next().unwrap().parse().unwrap(),
            z_start: split.next().unwrap().parse().unwrap(),
            z_end: split.next().unwrap().parse().unwrap(),
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

    /// Get the total length in the Z direction for this
    /// local segment as paraview would interpret it
    pub fn z_len(&self) -> usize {
        self.z_end - self.z_start + 1
    }

    /// Format the spans into a string that would be written to a vtk file
    pub(crate) fn to_string(&self) -> String {
        format!(
            "{} {} {} {} {} {}",
            self.x_start, self.x_end, self.y_start, self.y_end, self.z_start, self.z_end
        )
    }
}

impl Mesh<Binary> for Rectilinear3D<Binary> {
    // only write the headers here
    fn write_mesh_header<W: Write>(&self, writer: &mut EventWriter<W>) -> Result<(), Error> {
        let mut offset = 0;

        write_vtk::write_appended_dataarray_header(writer, "X", offset, 1)?;
        offset += (std::mem::size_of::<f64>() * (self.mesh.x_locations.len())) as i64;

        write_vtk::write_appended_dataarray_header(writer, "Y", offset, 1)?;
        offset += (std::mem::size_of::<f64>() * (self.mesh.y_locations.len())) as i64;

        write_vtk::write_appended_dataarray_header(writer, "Z", offset, 1)?;
        //offset += (std::mem::size_of::<f64>() * (self.z_locations.len())) as i64;
        //
        Ok(())
    }

    //
    fn write_mesh_appended<W: Write>(&self, writer: &mut EventWriter<W>) -> Result<(), Error> {
        self.mesh.x_locations.write_binary(writer)?;
        self.mesh.y_locations.write_binary(writer)?;
        self.mesh.z_locations.write_binary(writer)?;
        Ok(())
    }

    fn span_string(&self) -> String {
        self.spans.to_string()
    }

    fn mesh_bytes(&self) -> usize {
        let mut offset = 0;

        offset += std::mem::size_of::<f64>() * (self.mesh.x_locations.len());
        offset += std::mem::size_of::<f64>() * (self.mesh.y_locations.len());
        offset += std::mem::size_of::<f64>() * (self.mesh.z_locations.len());

        offset
    }
}

impl <T> ParseMesh for Mesh3D<T> {
    type Visitor = Mesh3DVisitor;
}

pub struct Mesh3DVisitor {
    x_locations: parse::PartialDataArrayBuffered,
    y_locations: parse::PartialDataArrayBuffered,
    z_locations: parse::PartialDataArrayBuffered,
}

impl Visitor<Spans3D> for Mesh3DVisitor {
    type Output = Mesh3D<Binary>;

    fn read_headers<'a>(spans: &Spans3D, buffer: &'a [u8]) -> IResult<&'a [u8], Self> {
        let (rest, x) = parse::parse_dataarray_or_lazy(buffer, b"X", spans.x_len())?;
        let (rest, y) = parse::parse_dataarray_or_lazy(rest, b"Y", spans.y_len())?;
        let (rest, z) = parse::parse_dataarray_or_lazy(rest, b"Z", spans.z_len())?;

        let x_locations = parse::PartialDataArrayBuffered::new(x, spans.x_len());
        let y_locations = parse::PartialDataArrayBuffered::new(y, spans.y_len());
        let z_locations = parse::PartialDataArrayBuffered::new(z, spans.z_len());

        let visitor = Self { x_locations, y_locations, z_locations };
        
        Ok((rest, visitor))
    }

    fn add_to_appended_reader<'a, 'b>(&'a self, buffer: &'b mut Vec<RefMut<'a, parse::OffsetBuffer>>) {
        self.x_locations.append_to_reader_list(buffer);
        self.y_locations.append_to_reader_list(buffer);
        self.z_locations.append_to_reader_list(buffer);
    }

    fn finish(self) -> Result<Self::Output, ParseError> {
        todo!();
    }
}
