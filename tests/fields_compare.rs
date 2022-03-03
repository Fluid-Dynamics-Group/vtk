#[cfg(feature = "derive")]
mod field3d {
    use vtk::prelude::*;

    use vtk::Mesh3D;
    use vtk::Rectilinear3D;
    use vtk::Spans3D;

    #[derive(vtk::DataArray, vtk::ParseArray, Debug, Clone)]
    #[vtk_parse(spans = "vtk::Spans3D")]
    pub struct SimpleArray {
        array: vtk::Field3D,
    }

    fn setup_vtk() -> VtkData<Rectilinear3D<vtk::Binary>, SimpleArray> {
        let x_locations = vec![0.0, 1.0, 2.0];
        let y_locations = vec![0.0, 1.0, 2.0];
        let z_locations = vec![0.0, 1.0, 2.0];
        let mesh = Mesh3D::new(x_locations, y_locations, z_locations);

        let spans = Spans3D::new(3, 3, 3);

        let data = vec![
            0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, //
            0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, //
            0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, //
            //
            //
            0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, //
            0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, //
            0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, //
            //
            //
            0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, //
            0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, //
            0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, //
        ];

        assert_eq!(3 * 3 * 3 * 3, data.len());

        let arr = ndarray::Array4::<f64>::from_shape_vec((3, 3, 3, 3), data).unwrap();

        dbg!(arr[[0, 0, 0, 0]], arr[[0, 0, 0, 1]], arr[[0, 0, 0, 2]],);

        let data = SimpleArray {
            array: vtk::Field3D::new(arr),
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

        let out_vtk: vtk::VtkData<Rectilinear3D<vtk::Binary>, SimpleArray> =
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
        array: vtk::Field2D,
    }

    fn setup_vtk() -> VtkData<Rectilinear2D<vtk::Ascii>, SimpleArray> {
        let x_locations = vec![0.0, 1.0, 2.0, 3.];
        let y_locations = vec![0.0, 1.0, 2.0, 3.];
        let mesh = Mesh2D::new(x_locations, y_locations);

        let nn = 3;
        let nx = 4;
        let ny = 4;

        let spans = Spans2D::new(4, 4);

        let arr: ndarray::Array3<f64> = ndarray::Array1::range(0., (nx * ny * nn) as f64, 1.)
            .into_shape((nn, nx, ny))
            .unwrap();

        assert_eq!(nx * ny * nn, arr.len());

        let data = SimpleArray {
            array: vtk::Field2D::new(arr),
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

        let out_vtk: vtk::VtkData<Rectilinear2D<vtk::Binary>, SimpleArray> =
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
        array: vtk::Scalar3D,
    }

    fn setup_vtk() -> VtkData<Rectilinear3D<vtk::Ascii>, SimpleArray> {
        let x_locations = vec![0.0, 1.0, 2.0, 3.];
        let y_locations = vec![0.0, 1.0, 2.0, 3.];
        let z_locations = vec![0.0, 1.0, 2.0, 3.];
        let mesh = Mesh3D::new(x_locations, y_locations, z_locations);

        let nx = 4;
        let ny = 4;
        let nz = 4;

        let spans = Spans3D::new(4, 4, 4);

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

        let out_vtk: vtk::VtkData<Rectilinear3D<vtk::Binary>, SimpleArray> =
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
        array: vtk::Scalar2D,
    }

    fn setup_vtk() -> VtkData<Rectilinear2D<vtk::Ascii>, SimpleArray> {
        let x_locations = vec![0.0, 1.0, 2.0, 3.];
        let y_locations = vec![0.0, 1.0, 2.0, 3.];
        let mesh = Mesh2D::new(x_locations, y_locations);

        let nx = 4;
        let ny = 4;

        let spans = Spans2D::new(4, 4);

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

        let out_vtk: vtk::VtkData<Rectilinear2D<vtk::Binary>, SimpleArray> =
            vtk::parse::parse_xml_document(&file).unwrap();
        let out_data = out_vtk.data;

        assert_eq!(data.array, out_data.array);
    }
}
