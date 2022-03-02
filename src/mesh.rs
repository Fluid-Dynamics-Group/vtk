use crate::traits::*;
use crate::write_vtk;
use crate::Binary;
use crate::Error;
use crate::EventWriter;
use crate::Visitor;
use crate::ParseError;
use crate::ParseMesh;
use std::io::Write;

pub struct Rectilinear3D {
    pub spans: Spans3D,
    pub mesh: Mesh3D
}

/// The X/Y/Z point data locations for the data points in the field
#[derive(Debug, Clone, Default, derive_builder::Builder, PartialEq)]
pub struct Mesh3D {
    pub x_locations: Vec<f64>,
    pub y_locations: Vec<f64>,
    pub z_locations: Vec<f64>,
}

/// The local locations
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
    pub fn new(span_string: &str) -> Self {
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

impl Mesh<Binary> for Rectilinear3D {
    // only write the headers here
    fn write_mesh_header<W: Write>(&self, writer: &mut EventWriter<W>) -> Result<(), Error> {
        let mut offset = 0;

        write_vtk::write_appended_dataarray_header(writer, "X", offset, 1)?;
        offset += (std::mem::size_of::<f64>() * (self.mesh.x_locations.len())) as i64;

        write_vtk::write_appended_dataarray_header(writer, "Y", offset, 1)?;
        offset += (std::mem::size_of::<f64>() * (self.mesh.y_locations.len())) as i64;

        write_vtk::write_appended_dataarray_header(writer, "Z", offset, 1)?;
        offset += (std::mem::size_of::<f64>() * (self.mesh.z_locations.len())) as i64;

        println!("mesh information covered {} bytes", offset);

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

        println!("reporting {} bytes from function call mesh_bytes", offset);
        offset
    }
}

impl ParseMesh for Mesh3D {
    type Visitor = Mesh3DVisitor;
}

struct Mesh3DVisitor {
    x_locations: parse::PartialDataArray,
    y_locations: parse::PartialDataArray,
    z_locations: parse::PartialDataArray,
}

impl Visitor<Spans3D> for Mesh3DVisitor {
    type Output = Mesh3D;

    fn read_headers(&mut self, spans: &Spans3D) -> Result<(), ParseError> {
        
        Ok(())
    }

    fn read_apppended(&mut self) -> Result<(), ParseError> {
        
        Ok(())
    }

    fn finish(self) -> Result<Self::Output, ParseError> {
        todo!();
    }
}
