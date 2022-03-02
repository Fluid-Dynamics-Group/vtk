#[derive(Debug, Default, Clone, PartialEq)]
pub struct VtkData<DOMAIN, D> {
    pub domain: DOMAIN,
    pub data: D,
}

impl<DOMAIN, D> VtkData<DOMAIN, D> {
    pub fn new_data<T>(self, new_data: T) -> VtkData<DOMAIN, T> {
        VtkData {
            domain: self.domain,
            data: new_data,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::helpers::SpanData;

    use crate as vtk;
    use crate::Array;

    #[test]
    fn data_add() {
        let data = SpanData {
            u: vec![0., 1., 2.],
        };
        let data_2 = SpanData {
            u: vec![0., 0., 1.],
        };
        let expected = SpanData {
            u: vec![0., 1., 3.],
        };

        assert_eq!(data + data_2, expected)
    }

    #[test]
    fn data_div() {
        let data = SpanData {
            u: vec![3., 3., 3.],
        };
        let expected = SpanData {
            u: vec![1., 1., 1.],
        };

        assert_eq!(data / 3., expected)
    }

    #[derive(crate::DataArray, crate::ParseDataArray, Debug, Clone)]
    struct SimpleArray {
        array: ndarray::Array4<f64>,
    }

    fn setup_vtk() -> VtkData<SimpleArray> {
        let x_locations = vec![0.0, 1.0, 2.0];
        let y_locations = vec![0.0, 1.0, 2.0];
        let z_locations = vec![0.0, 1.0, 2.0];
        let locations = Locations {
            x_locations,
            y_locations,
            z_locations,
        };

        let spans = LocationSpans {
            x_start: 1,
            x_end: 3,
            y_start: 1,
            y_end: 3,
            z_start: 1,
            z_end: 3,
        };

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
        // TODO: i dont remember why this call is here
        let arr = arr.reversed_axes();

        dbg!(arr[[0, 0, 0, 0]], arr[[0, 0, 0, 1]], arr[[0, 0, 0, 2]],);

        let data = SimpleArray { array: arr };

        dbg!(&data);

        crate::VtkData {
            data,
            spans,
            locations,
        }
    }

    #[test]
    fn write_simple_array() {
        let vtk = setup_vtk();

        let file = std::fs::File::create("./test_vtks/simple_vector_array.vtk").unwrap();
        vtk::write_vtk(file, vtk, true).unwrap();
    }

    #[test]
    fn read_simple_vtk_after_write() {
        let mut file = Vec::new();
        let vtk = setup_vtk();
        let data = vtk.data.clone();
        vtk::write_vtk(&mut file, vtk, true).unwrap();

        let out_vtk = crate::parse::parse_xml_document::<SimpleArray>(&file).unwrap();
        let out_data = out_vtk.data;

        assert_eq!(data.array, out_data.array);
    }
}
