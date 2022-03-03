#[derive(Debug, Default, Clone, PartialEq)]
pub struct VtkData<DOMAIN, D> {
    pub domain: DOMAIN,
    pub data: D,
}

impl<DOMAIN, D> VtkData<DOMAIN, D> {
    /// Construct a `vtk` container for writing to a file
    pub fn new(domain: DOMAIN, data: D) -> VtkData<DOMAIN, D> {
        VtkData { domain, data }
    }

    /// change the datatype of the data stored in this container
    pub fn new_data<T>(self, new_data: T) -> VtkData<DOMAIN, T> {
        VtkData {
            domain: self.domain,
            data: new_data,
        }
    }
}

//#[cfg(test)]
//mod tests {
//    use super::*;
//    use crate::helpers::SpanData;
//
//    use crate as vtk;
//    use crate::Array;
//    use vtk::Mesh3D;
//    use vtk::Rectilinear3D;
//    use vtk::Spans3D;
//
//    #[derive(crate::DataArray, crate::ParseArray, Debug, Clone)]
//    #[vtk_parse(spans = "vtk::Spans3D")]
//    pub struct SimpleArray {
//        array: ndarray::Array4<f64>,
//    }
//
//    fn setup_vtk() -> VtkData<Rectilinear3D<vtk::Binary>, SimpleArray> {
//        let x_locations = vec![0.0, 1.0, 2.0];
//        let y_locations = vec![0.0, 1.0, 2.0];
//        let z_locations = vec![0.0, 1.0, 2.0];
//        let mesh = Mesh3D::new(x_locations, y_locations, z_locations);
//
//        let spans = Spans3D::new(3, 3, 3);
//
//        let data = vec![
//            0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, //
//            0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, //
//            0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, //
//            //
//            //
//            0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, //
//            0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, //
//            0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, //
//            //
//            //
//            0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, //
//            0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, //
//            0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, //
//        ];
//
//        assert_eq!(3 * 3 * 3 * 3, data.len());
//
//        let arr = ndarray::Array4::<f64>::from_shape_vec((3, 3, 3, 3), data).unwrap();
//        // TODO: i dont remember why this call is here
//        let arr = arr.reversed_axes();
//
//        dbg!(arr[[0, 0, 0, 0]], arr[[0, 0, 0, 1]], arr[[0, 0, 0, 2]],);
//
//        let data = SimpleArray { array: arr };
//        let domain = Rectilinear3D::new(mesh, spans);
//
//        dbg!(&data);
//
//        crate::VtkData { data, domain }
//    }
//
//    #[test]
//    fn write_simple_array() {
//        let vtk = setup_vtk();
//
//        let file = std::fs::File::create("./test_vtks/simple_vector_array.vtk").unwrap();
//        vtk::write_vtk(file, vtk).unwrap();
//    }
//
//    #[test]
//    fn read_simple_vtk_after_write() {
//        let mut file = Vec::new();
//        let vtk = setup_vtk();
//        let data = vtk.data.clone();
//        vtk::write_vtk(&mut file, vtk).unwrap();
//
//        let out_vtk: vtk::VtkData<Rectilinear3D<vtk::Binary>, SimpleArray> =
//            crate::parse::parse_xml_document(&file).unwrap();
//        let out_data = out_vtk.data;
//
//        assert_eq!(data.array, out_data.array);
//    }
//}
