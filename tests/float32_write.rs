#[cfg(all(test, feature = "derive"))]
mod inner {
    use vtk::Field3D;
    use vtk::Mesh3D;
    use vtk::Rectilinear3D;
    use vtk::Scalar3D;
    use vtk::Spans3D;

    use ndarray::Array1;
    use ndarray::Array3;
    use ndarray::Array4;

    #[derive(vtk::DataArray, Clone)]
    #[vtk_write(encoding = "binary")]
    pub struct Binary {
        rho: Scalar3D<f32>,
        velocity: Field3D<f32>,
    }

    #[derive(vtk::DataArray, Clone)]
    #[vtk_write(encoding = "base64")]
    pub struct Base64 {
        rho: Scalar3D<f32>,
        velocity: Field3D<f32>,
    }

    #[derive(vtk::DataArray, Clone)]
    #[vtk_write(encoding = "ascii")]
    pub struct Ascii {
        rho: Scalar3D<f32>,
        velocity: Field3D<f32>,
    }

    const NX: usize = 10;
    const NY: usize = NX;
    const NZ: usize = NX;

    fn generate_data<ENC>() -> (Scalar3D<f32>, Field3D<f32>, Mesh3D<f32, ENC>, Spans3D) {
        let mut rho = Array3::<f32>::zeros((NX, NY, NZ));
        let mut velocity = Array4::<f32>::zeros((3, NX, NY, NZ));

        let x: Array1<f32> = ndarray::ArrayBase::linspace(0., 2. * 3.14, NX);
        let y: Array1<f32> = ndarray::ArrayBase::linspace(0., 2. * 3.14, NY);
        let z: Array1<f32> = ndarray::ArrayBase::linspace(0., 2. * 3.14, NZ);

        for i in 0..NX {
            for j in 0..NY {
                for k in 0..NZ {
                    rho[[i, j, k]] = x[i].sin();

                    for v in 0..3 {
                        let val = if v == 0 {
                            x[[i]]
                        } else if v == 1 {
                            y[[j]]
                        } else {
                            z[[k]]
                        };

                        velocity[[v, i, j, k]] = val.sin();
                    }
                }
            }
        }

        (
            Scalar3D::new(rho),
            Field3D::new(velocity),
            Mesh3D::new(x.to_vec(), y.to_vec(), z.to_vec()),
            Spans3D::new(NX, NY, NZ),
        )
    }

    #[test]
    fn write_binary() {
        let path = "./test_vtks/f32_binary.vtr";
        let file = std::fs::File::create(path).unwrap();
        let writer = std::io::BufWriter::new(file);

        let (rho, velocity, mesh, spans) = generate_data::<vtk::Binary>();
        let data = Binary { rho, velocity };
        let domain: vtk::Rectilinear3D<f32, vtk::Binary> = vtk::Rectilinear3D::new(mesh, spans);

        let _vtk = vtk::VtkData::new(domain, data);

        vtk::write_vtk(writer, _vtk).unwrap();
    }

    #[test]
    fn write_base64() {
        let path = "./test_vtks/f32_base64.vtr";
        let file = std::fs::File::create(path).unwrap();
        let writer = std::io::BufWriter::new(file);

        let (rho, velocity, mesh, spans) = generate_data::<vtk::Ascii>();
        let data = Base64 { rho, velocity };
        let domain: vtk::Rectilinear3D<f32, vtk::Ascii> = vtk::Rectilinear3D::new(mesh, spans);

        let _vtk = vtk::VtkData::new(domain, data);

        vtk::write_vtk(writer, _vtk).unwrap();
    }

    #[test]
    fn write_ascii() {
        let path = "./test_vtks/f32_ascii.vtr";
        let file = std::fs::File::create(path).unwrap();
        let writer = std::io::BufWriter::new(file);

        let (rho, velocity, mesh, spans) = generate_data::<vtk::Ascii>();
        let data = Ascii { rho, velocity };
        let domain: vtk::Rectilinear3D<f32, vtk::Ascii> = vtk::Rectilinear3D::new(mesh, spans);

        let _vtk = vtk::VtkData::new(domain, data);

        vtk::write_vtk(writer, _vtk).unwrap();
    }
}
