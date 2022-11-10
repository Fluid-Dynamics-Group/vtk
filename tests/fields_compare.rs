#[cfg(feature = "derive")]
mod field3d {
    use vtk::prelude::*;

    use vtk::Mesh3D;
    use vtk::Rectilinear3D;
    use vtk::Spans3D;

    #[derive(vtk::DataArray, vtk::ParseArray, Debug, Clone)]
    #[vtk_parse(spans = "vtk::Spans3D")]
    #[vtk_write(encoding = "binary")]
    pub struct SimpleArray {
        array: vtk::Vector3D<f64>,
    }

    fn setup_vtk() -> VtkData<Rectilinear3D<f64, vtk::Binary>, SimpleArray> {
        let nx = 2;
        let ny = 3;
        let nz = 4;

        let nn = 3;

        let x_locations: Vec<f64> = ndarray::Array1::linspace(0., 1., nx).to_vec();
        let y_locations: Vec<f64> = ndarray::Array1::linspace(0., 1., ny).to_vec();
        let z_locations: Vec<f64> = ndarray::Array1::linspace(0., 1., nz).to_vec();
        let mesh = Mesh3D::new(x_locations, y_locations, z_locations);

        let spans = Spans3D::new(nx, ny, nz);

        dbg!(&mesh);
        dbg!(&spans);

        let arr: ndarray::Array4<f64> = ndarray::Array1::range(0., (nx * ny * nn * nz) as f64, 1.)
            .into_shape((nn, nx, ny, nz))
            .unwrap();

        assert_eq!(nx * ny * nn * nz, arr.len());

        let data = SimpleArray {
            array: vtk::Vector3D::new(arr),
        };
        let domain = Rectilinear3D::new(mesh, spans);

        dbg!(&data);

        vtk::VtkData { data, domain }
    }

    #[test]
    fn write_simple_array() {
        let vtk = setup_vtk();

        let file = std::fs::File::create("./test_vtks/simple_vector_array_field_3d.vtr").unwrap();

        vtk::write_vtk(file, vtk).unwrap();
    }

    #[test]
    fn read_simple_vtk_after_write() {
        let mut file = Vec::new();
        let vtk = setup_vtk();
        let data = vtk.data.clone();
        vtk::write_vtk(&mut file, vtk).unwrap();

        let out_vtk: vtk::VtkData<Rectilinear3D<f64, vtk::Binary>, SimpleArray> =
            vtk::parse::parse_xml_document(&file).unwrap();
        let out_data = out_vtk.data;

        assert_eq!(data.array, out_data.array);
    }
}

#[cfg(feature = "derive")]
mod field2d {
    use vtk::prelude::*;

    use vtk::Mesh2D;
    use vtk::Rectilinear2D;
    use vtk::Spans2D;

    #[derive(vtk::DataArray, vtk::ParseArray, Debug, Clone)]
    #[vtk_parse(spans = "vtk::Spans2D")]
    #[vtk_write(encoding = "binary")]
    pub struct SimpleArray {
        array: vtk::Vector2D<f64>,
    }

    fn setup_vtk() -> VtkData<Rectilinear2D<f64, vtk::Binary>, SimpleArray> {
        let nx = 4;
        let ny = 4;
        let nn = 3;

        let x_locations: Vec<f64> = ndarray::Array1::linspace(0., 1., nx).to_vec();
        let y_locations: Vec<f64> = ndarray::Array1::linspace(0., 1., ny).to_vec();
        let mesh = Mesh2D::new(x_locations, y_locations);

        let spans = Spans2D::new(nx, ny);

        let arr: ndarray::Array3<f64> = ndarray::Array1::range(0., (nx * ny * nn) as f64, 1.)
            .into_shape((nn, nx, ny))
            .unwrap();

        assert_eq!(nx * ny * nn, arr.len());

        assert_eq!(mesh.x_locations.len(), spans.x_len());
        assert_eq!(mesh.y_locations.len(), spans.y_len());

        let data = SimpleArray {
            array: vtk::Vector2D::new(arr),
        };

        let domain = Rectilinear2D::new(mesh, spans);

        dbg!(&data);

        vtk::VtkData { data, domain }
    }

    #[test]
    fn write_simple_array() {
        let vtk = setup_vtk();

        let file = std::fs::File::create("./test_vtks/simple_vector_array_field_2d.vtr").unwrap();
        vtk::write_vtk(file, vtk.clone()).unwrap();

        let (nn, nx, ny) = vtk.data.array.dim();

        for j in 0..ny {
            for i in 0..nx {
                for n in 0..nn {
                    let value = vtk.data.array.get((n, i, j)).unwrap();
                    println!("({i},{j}) @ {n} -> {value}")
                }
            }
        }
    }

    #[test]
    fn read_simple_vtk_after_write() {
        let mut file = Vec::new();
        let vtk = setup_vtk();
        let data = vtk.data.clone();
        vtk::write_vtk(&mut file, vtk).unwrap();

        let out_vtk: vtk::VtkData<Rectilinear2D<f64, vtk::Binary>, SimpleArray> =
            vtk::parse::parse_xml_document(&file).unwrap();
        let out_data = out_vtk.data;

        assert_eq!(data.array, out_data.array);
    }
}

#[cfg(feature = "derive")]
mod scalar_3d {
    use vtk::prelude::*;

    use vtk::Mesh3D;
    use vtk::Rectilinear3D;
    use vtk::Spans3D;

    #[derive(vtk::DataArray, vtk::ParseArray, Debug, Clone)]
    #[vtk_parse(spans = "vtk::Spans3D")]
    #[vtk_write(encoding = "binary")]
    pub struct SimpleArray {
        array: vtk::Scalar3D<f64>,
    }

    fn setup_vtk() -> VtkData<Rectilinear3D<f64, vtk::Binary>, SimpleArray> {
        let nx = 2;
        let ny = 4;
        let nz = 5;

        let x_locations: Vec<f64> = ndarray::Array1::linspace(0., 1., nx).to_vec();
        let y_locations: Vec<f64> = ndarray::Array1::linspace(0., 1., ny).to_vec();
        let z_locations: Vec<f64> = ndarray::Array1::linspace(0., 1., nz).to_vec();

        let mesh = Mesh3D::new(x_locations, y_locations, z_locations);

        let spans = Spans3D::new(nx, ny, nz);

        let arr: ndarray::Array3<f64> = ndarray::Array1::range(0., (nx * ny * nz) as f64, 1.)
            .into_shape((nx, ny, nz))
            .unwrap();

        assert_eq!(nx * ny * nz, arr.len());

        let data = SimpleArray {
            array: vtk::Scalar3D::new(arr),
        };
        let domain = Rectilinear3D::new(mesh, spans);

        dbg!(&data);

        vtk::VtkData { data, domain }
    }

    #[test]
    fn write_simple_array() {
        let vtk = setup_vtk();

        let file = std::fs::File::create("./test_vtks/simple_vector_array_scalar_3d.vtr").unwrap();
        vtk::write_vtk(file, vtk.clone()).unwrap();

        let (nn, nx, ny) = vtk.data.array.dim();

        for j in 0..ny {
            for i in 0..nx {
                for n in 0..nn {
                    let value = vtk.data.array.get((n, i, j)).unwrap();
                    println!("{value}")
                }
            }
        }
    }

    #[test]
    fn read_simple_vtk_after_write() {
        let mut file = Vec::new();
        let vtk = setup_vtk();
        let data = vtk.data.clone();
        vtk::write_vtk(&mut file, vtk).unwrap();

        let out_vtk: vtk::VtkData<Rectilinear3D<f64, vtk::Binary>, SimpleArray> =
            vtk::parse::parse_xml_document(&file).unwrap();
        let out_data = out_vtk.data;

        assert_eq!(data.array, out_data.array);
    }
}

#[cfg(feature = "derive")]
mod scalar_2d {
    use vtk::prelude::*;

    use vtk::Mesh2D;
    use vtk::Rectilinear2D;
    use vtk::Spans2D;

    #[derive(vtk::DataArray, vtk::ParseArray, Debug, Clone)]
    #[vtk_parse(spans = "vtk::Spans2D")]
    #[vtk_write(encoding = "binary")]
    pub struct SimpleArray {
        array: vtk::Scalar2D<f64>,
    }

    fn setup_vtk() -> VtkData<Rectilinear2D<f64, vtk::Binary>, SimpleArray> {
        let nx = 3;
        let ny = 4;

        let x_locations: Vec<f64> = ndarray::Array1::linspace(0., 1., nx).to_vec();
        let y_locations: Vec<f64> = ndarray::Array1::linspace(0., 1., ny).to_vec();

        let mesh = Mesh2D::new(x_locations, y_locations);

        let spans = Spans2D::new(nx, ny);

        let arr: ndarray::Array2<f64> = ndarray::Array1::range(0., (nx * ny) as f64, 1.)
            .into_shape((nx, ny))
            .unwrap();

        assert_eq!(nx * ny, arr.len());

        let data = SimpleArray {
            array: vtk::Scalar2D::new(arr),
        };
        let domain = Rectilinear2D::new(mesh, spans);

        dbg!(&data);

        vtk::VtkData { data, domain }
    }

    #[test]
    fn write_simple_array() {
        let vtk = setup_vtk();

        let file = std::fs::File::create("./test_vtks/simple_vector_array_scalar_2d.vtr").unwrap();
        vtk::write_vtk(file, vtk.clone()).unwrap();
    }

    #[test]
    fn read_simple_vtk_after_write() {
        let mut file = Vec::new();
        let vtk = setup_vtk();
        let data = vtk.data.clone();
        vtk::write_vtk(&mut file, vtk).unwrap();

        let out_vtk: vtk::VtkData<Rectilinear2D<f64, vtk::Binary>, SimpleArray> =
            vtk::parse::parse_xml_document(&file).unwrap();
        let out_data = out_vtk.data;

        dbg!(data.array.shape());
        dbg!(out_data.array.shape());

        assert_eq!(data.array, out_data.array);
    }
}
