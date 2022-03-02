
#[cfg(all(test, feature = "derive"))]
mod inner {
    use vtk::Mesh3D;
    use vtk::Spans3D;
    use vtk::Rectilinear3D;

    #[derive(vtk::DataArray, Clone, Debug, vtk::ParseArray)]
    #[vtk_write(encoding = "binary")]
    #[vtk_parse(spans="vtk::Spans3D")]
    struct Binary {
        rho: Vec<f64>,
        u: Vec<f64>,
        v: Vec<f64>,
        w: Vec<f64>,
    }

    #[derive(vtk::DataArray, Clone, vtk::ParseArray)]
    #[vtk_write(encoding = "base64")]
    #[vtk_parse(spans="vtk::Spans3D")]
    struct Base64 {
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

    #[test]
    fn inline_ascii_points_appended_binary_data() {
        let data = create_data();
        let mut writer = Vec::new();
        vtk::write_vtk(&mut writer, data.clone()).unwrap();

        let output_data: vtk::VtkData<Rectilinear3D<vtk::Binary>, Binary> =
            vtk::parse::parse_xml_document(writer.as_slice()).unwrap();

        assert_eq!(output_data.domain.spans, data.domain.spans);
        assert_eq!(output_data.domain.mesh, data.domain.mesh);
        assert_eq!(output_data.data.rho, data.data.rho);
        assert_eq!(output_data.data.u, data.data.u);
        assert_eq!(output_data.data.v, data.data.v);
        assert_eq!(output_data.data.w, data.data.w);
    }

    #[test]
    fn appended_ascii_points_appended_binary_data() {
        let data = create_data();
        let mut writer = Vec::new();
        vtk::write_vtk(&mut writer, data.clone()).unwrap();

        let output_data: vtk::VtkData<Rectilinear3D<vtk::Binary>, Binary> =
            vtk::parse::parse_xml_document(writer.as_slice()).unwrap();

        assert_eq!(output_data.domain.spans, data.domain.spans);
        assert_eq!(output_data.domain.mesh, data.domain.mesh);
        assert_eq!(output_data.data.rho, data.data.rho);
        assert_eq!(output_data.data.u, data.data.u);
        assert_eq!(output_data.data.v, data.data.v);
        assert_eq!(output_data.data.w, data.data.w);
    }

    #[test]
    fn inline_points_inline_base64() {
        let data = create_data();
        let mut writer = Vec::new();
        let mesh = data.domain.mesh.clone();

        //let data = data.data.clone();

        let base64 = data.new_data(Base64::from(data.data.clone()));

        vtk::write_vtk(&mut writer, base64.clone()).unwrap();

        let output_data: vtk::VtkData<Rectilinear3D<vtk::Binary>, Base64> =
            vtk::parse::parse_xml_document(writer.as_slice()).unwrap();

        assert_eq!(output_data.domain.spans, base64.domain.spans);
        assert_eq!(output_data.domain.mesh, base64.domain.mesh);
        assert_eq!(output_data.data.rho, base64.data.rho);
        assert_eq!(output_data.data.u, base64.data.u);
        assert_eq!(output_data.data.v, base64.data.v);
        assert_eq!(output_data.data.w, base64.data.w);
    }
}
