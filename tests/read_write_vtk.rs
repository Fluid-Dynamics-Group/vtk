#[cfg(all(test, feature = "derive"))]
mod inner {
    use vtk::Mesh3D;
    use vtk::Rectilinear3D;
    use vtk::Spans3D;

    #[derive(vtk::DataArray, Clone, Debug, vtk::ParseArray)]
    #[vtk_write(encoding = "binary")]
    #[vtk_parse(spans = "vtk::Spans3D")]
    pub struct Binary {
        rho: Vec<f64>,
        u: Vec<f64>,
        v: Vec<f64>,
        w: Vec<f64>,
    }

    #[derive(vtk::DataArray, Clone, vtk::ParseArray)]
    #[vtk_write(encoding = "base64")]
    #[vtk_parse(spans = "vtk::Spans3D")]
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
        fn from(x: Base64) -> Self {
            let Base64 { rho, u, v, w } = x;
            Binary { rho, u, v, w }
        }
    }

    fn create_data() -> vtk::VtkData<Rectilinear3D<f64, vtk::Ascii>, Binary> {
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

        let data = vtk::VtkData { domain, data };

        data
    }

    fn check_assertions<T, V>(
        left: vtk::VtkData<Rectilinear3D<f64, T>, Binary>,
        right: vtk::VtkData<Rectilinear3D<f64, V>, Binary>,
    ) where
        V: std::fmt::Debug,
        T: std::fmt::Debug,
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

        let string = String::from_utf8(writer.as_slice().to_vec()).unwrap();
        let reader = vtk::Reader::from_str(&string);

        let output_data: vtk::VtkData<Rectilinear3D<f64, vtk::Binary>, Binary> =
            vtk::parse::parse_xml_document(reader).unwrap();

        check_assertions(data, output_data);
    }

    #[test]
    fn appended_ascii_points_appended_binary_data() {
        let data = create_data();
        let mut writer = Vec::new();
        vtk::write_vtk(&mut writer, data.clone()).unwrap();

        let string = String::from_utf8(writer.as_slice().to_vec()).unwrap();
        let reader = vtk::Reader::from_str(&string);

        let output_data: vtk::VtkData<Rectilinear3D<f64, vtk::Binary>, Binary> =
            vtk::parse::parse_xml_document(reader).unwrap();

        check_assertions(data, output_data);
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

        let string = String::from_utf8(writer.as_slice().to_vec()).unwrap();
        let reader = vtk::Reader::from_str(&string);

        let output_data: vtk::VtkData<Rectilinear3D<f64, vtk::Binary>, Base64> =
            vtk::parse::parse_xml_document(reader).unwrap();

        let output_data_inner = output_data.data.clone();
        let output_in_binary = output_data.new_data(Binary::from(output_data_inner));

        check_assertions(vtk_data_c, output_in_binary);
    }

    #[derive(vtk::DataArray, vtk::ParseArray, Clone, PartialEq, Debug)]
    #[vtk_parse(spans = "vtk::Spans2D")]
    /// Information available from a span-wise average of the flowfield
    pub struct SpanVtkInformation2D {
        pub(crate) rho: vtk::Scalar2D<f64>,
        pub(crate) velocity: vtk::Vector2D<f64>,
        pub(crate) energy: vtk::Scalar2D<f64>,
    }

    #[test]
    fn read_write_binary_2d() {
        let nx = 800;
        let ny = 208;

        let rho = vtk::Scalar2D::new(ndarray::Array2::ones((nx, ny)));
        let velocity = vtk::Vector2D::new(ndarray::Array3::ones((3, nx, ny)));
        let energy = vtk::Scalar2D::new(ndarray::Array2::zeros((nx, ny)));

        let mesh_x: Vec<f64> = ndarray::Array1::linspace(0., 1., nx).to_vec();
        let mesh_y: Vec<f64> = ndarray::Array1::linspace(0., 1., ny).to_vec();

        assert_eq!(mesh_x.len(), nx);
        assert_eq!(mesh_y.len(), ny);
        let spans = vtk::Spans2D::new(nx, ny);
        let mesh = vtk::Mesh2D::<f64, vtk::Binary>::new(mesh_x, mesh_y);

        let span = SpanVtkInformation2D {
            rho,
            velocity,
            energy,
        };
        let domain = vtk::Rectilinear2D::new(mesh, spans);
        let vtk_data = vtk::VtkData::new(domain, span.clone());

        let mut buffer = Vec::new();
        vtk::write_vtk(&mut buffer, vtk_data).unwrap();

        let string = String::from_utf8(buffer).unwrap();
        let reader = vtk::Reader::from_str(&string);

        // now we parse the data back out
        let out: vtk::VtkData<vtk::Rectilinear2D<f64, vtk::Binary>, SpanVtkInformation2D> =
            vtk::parse::parse_xml_document(reader).unwrap();

        assert_eq!(out.data, span);
    }

    #[derive(vtk::DataArray, vtk::ParseArray, Clone, PartialEq, Debug)]
    #[vtk_parse(spans = "vtk::Spans3D")]
    /// Information available from a span-wise average of the flowfield
    pub struct SpanVtkInformation3D {
        pub(crate) rho: vtk::Scalar3D<f64>,
    }

    #[test]
    fn read_write_binary_3d() {
        let nx = 800;
        let ny = 208;
        let nz = 1;

        let rho = vtk::Scalar3D::new(ndarray::Array3::ones((nx, ny, 1)));

        let mesh_x: Vec<f64> = ndarray::Array1::linspace(0., 1., nx).to_vec();
        let mesh_y: Vec<f64> = ndarray::Array1::linspace(0., 1., ny).to_vec();
        let mesh_z: Vec<f64> = ndarray::Array1::linspace(0., 1., nz).to_vec();

        assert_eq!(mesh_x.len(), nx);
        assert_eq!(mesh_y.len(), ny);
        assert_eq!(mesh_z.len(), nz);

        let spans = vtk::Spans3D::new(nx, ny, nz);
        dbg!(&spans);
        let mesh = vtk::Mesh3D::<f64, vtk::Binary>::new(mesh_x, mesh_y, mesh_z);

        //let span = SpanVtkInformation { rho, velocity, energy };
        let span = SpanVtkInformation3D { rho };
        let domain = vtk::Rectilinear3D::new(mesh, spans);
        let vtk_data = vtk::VtkData::new(domain, span.clone());

        let mut buffer = Vec::new();
        vtk::write_vtk(&mut buffer, vtk_data).unwrap();

        let string = String::from_utf8(buffer).unwrap();
        let reader = vtk::Reader::from_str(&string);

        // now we parse the data back out
        let out: vtk::VtkData<vtk::Rectilinear3D<f64, vtk::Binary>, SpanVtkInformation3D> =
            vtk::parse::parse_xml_document(reader).unwrap();
        dbg!(out.data.rho.shape());
        dbg!(span.rho.shape());
        assert_eq!(out.data, span);
    }
}
