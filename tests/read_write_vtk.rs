
#[cfg(all(test, feature = "derive"))]
mod inner {
    use vtk::Mesh3D;
    use vtk::Spans3D;
    use vtk::Rectilinear3D;

    #[derive(vtk::DataArray, Clone, Debug, vtk::ParseArray)]
    #[vtk_write(encoding = "binary")]
    #[vtk_parse(spans="vtk::Spans3D")]
    pub struct Binary {
        rho: Vec<f64>,
        u: Vec<f64>,
        v: Vec<f64>,
        w: Vec<f64>,
    }

    #[derive(vtk::DataArray, Clone, vtk::ParseArray)]
    #[vtk_write(encoding = "base64")]
    #[vtk_parse(spans="vtk::Spans3D")]
    pub struct Base64 {
        rho: Vec<f64>,
        u: Vec<f64>,
        v: Vec<f64>,
        w: Vec<f64>,
    }

    impl From<Binary> for Base64 {
        fn from(x: Binary) -> Self {
            let Binary { rho, u, v, w } = x;
            Base64 { rho, u, v, w }
        }
    }

    impl From<Base64> for Binary {
        fn from(x: Base64 ) -> Self {
            let Base64 { rho, u, v, w } = x;
            Binary { rho, u, v, w }
        }
    }

    fn create_data() -> vtk::VtkData<Rectilinear3D<vtk::Ascii>, Binary> {
        let spans = Spans3D::new(5, 5, 5);

        let length = spans.x_len() * spans.y_len() * spans.z_len();

        let mesh = Mesh3D::new(
            vec![0., 1., 2., 3., 4.],
            vec![0., 1., 2., 3., 4.],
            vec![0., 1., 2., 3., 4.],
        );

        let rho: Vec<_> = std::iter::repeat(0)
            .take(length)
            .enumerate()
            .map(|(i, _)| i as f64)
            .collect();
        let u = std::iter::repeat(0)
            .take(length)
            .enumerate()
            .map(|(i, _)| i as f64)
            .collect();
        let v = std::iter::repeat(0)
            .take(length)
            .enumerate()
            .map(|(i, _)| i as f64)
            .collect();
        let w = std::iter::repeat(0)
            .take(length)
            .enumerate()
            .map(|(i, _)| i as f64)
            .collect();

        dbg!(rho.len());

        let data = Binary { rho, u, v, w };

        let domain = Rectilinear3D::new(mesh, spans);

        let data = vtk::VtkData { domain , data };

        data
    }

    fn check_assertions<T,V>(left: vtk::VtkData<Rectilinear3D<T>, Binary>, right: vtk::VtkData<Rectilinear3D<V>, Binary>) 
    where V: std::fmt::Debug,T: std::fmt::Debug,
  {
        assert_eq!(left.domain.spans, right.domain.spans);
        assert_eq!(left.domain.mesh, right.domain.mesh);
        assert_eq!(left.data.rho, right.data.rho);
        assert_eq!(left.data.u, right.data.u);
        assert_eq!(left.data.v, right.data.v);
        assert_eq!(left.data.w, right.data.w);
        
    }

    #[test]
    fn inline_ascii_points_appended_binary_data() {
        let data = create_data();
        let mut writer = Vec::new();
        vtk::write_vtk(&mut writer, data.clone()).unwrap();

        let output_data: vtk::VtkData<Rectilinear3D<vtk::Binary>, Binary> =
            vtk::parse::parse_xml_document(writer.as_slice()).unwrap();

        check_assertions(data,output_data);
    }

    #[test]
    fn appended_ascii_points_appended_binary_data() {
        let data = create_data();
        let mut writer = Vec::new();
        vtk::write_vtk(&mut writer, data.clone()).unwrap();

        let output_data: vtk::VtkData<Rectilinear3D<vtk::Binary>, Binary> =
            vtk::parse::parse_xml_document(writer.as_slice()).unwrap();

        check_assertions(data,output_data);
    }

    #[test]
    fn inline_points_inline_base64() {
        let vtk_data = create_data();
        let vtk_data_c = vtk_data.clone();
        let mut writer = Vec::new();
        //let mesh = vtk_data.domain.mesh.clone();

        let inner_data = vtk_data.data.clone();
        let base64 = vtk_data.new_data(Base64::from(inner_data));

        vtk::write_vtk(&mut writer, base64.clone()).unwrap();

        let output_data: vtk::VtkData<Rectilinear3D<vtk::Binary>, Base64> =
            vtk::parse::parse_xml_document(writer.as_slice()).unwrap();

        let output_data_inner = output_data.data.clone();
        let output_in_binary = output_data.new_data(Binary::from(output_data_inner));

        check_assertions(vtk_data_c, output_in_binary);
    }
}
