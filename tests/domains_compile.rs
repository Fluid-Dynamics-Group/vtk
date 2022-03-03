use vtk::prelude::*;
use vtk::Spans3D;
use vtk::Mesh3D;
use vtk::Rectilinear3D;
use vtk::Binary;
use vtk::parse;
use vtk::ParseError;
use nom::IResult;
use std::cell::RefMut;


#[cfg(test)]
struct ArrayContainer;

#[cfg(test)]
struct ArrayContainerVisitor;

#[cfg(test)]
impl ParseArray for ArrayContainer {
    type Visitor = ArrayContainerVisitor;
}

impl DataArray<vtk::Ascii> for ArrayContainer {
    fn write_array_header<W: std::io::Write>(
        &self,
        _writer: &mut EventWriter<W>,
        _starting_offset: i64,
    ) -> Result<(), vtk::Error> {
        Ok(())
    }

    fn write_array_appended<W: std::io::Write>(
        &self,
        _writer: &mut EventWriter<W>,
    ) -> Result<(), vtk::Error> {
        
        Ok(())
    }
    
}

#[cfg(test)]
impl <T> Visitor<T> for ArrayContainerVisitor {
    type Output = ArrayContainer;

    fn read_headers<'a>(_spans: &T, _buffer: &'a [u8]) -> IResult<&'a [u8], Self> {
        Ok((_buffer, ArrayContainerVisitor))
    }

    fn add_to_appended_reader<'a, 'b>(
        &'a self,
        _buffer: &'b mut Vec<RefMut<'a, parse::OffsetBuffer>>,
    ) { }

    fn finish(self, _: &T) -> Result<Self::Output, ParseError> {
        Ok(ArrayContainer)
    }
}


#[test]
/// verify we have implemented all the traits for Rectilinear3D to write files
fn compile_dim3_write() {
    let arrays = ArrayContainer;

    let mesh = Mesh3D::<vtk::Ascii>::new(vec![], vec![], vec![]);
    let spans = Spans3D::new(1,1,1);
    let domain = Rectilinear3D::new(mesh, spans);
    let vtk = VtkData::new(domain, arrays);

    let writer = Vec::new();

    vtk::write_vtk(writer, vtk).ok();
}

#[test]
/// verify we have implemented all the traits for Rectilinear3D to read files
fn compile_dim3_read() {
    let path = std::path::PathBuf::from("/");

    let _ : Result<VtkData<Rectilinear3D<vtk::Binary>, ArrayContainer>, _> = vtk::read_vtk(&path);
}
