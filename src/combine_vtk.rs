use super::data::{LocationSpans, Locations, VtkData};
use super::traits::Combine;

/// utility function for combining vtk information from different subsections
/// of a flowfield.
pub fn combine_vtk<T: Combine, D: From<T>>(data: T) -> VtkData<D> {
    let x_locations = data.x_locations();
    let y_locations = data.x_locations();
    let z_locations = data.x_locations();

    let (x_start, x_end) = data.x_dims();
    let (y_start, y_end) = data.y_dims();
    let (z_start, z_end) = data.z_dims();

    let spans = LocationSpans {
        x_start,
        x_end,
        y_start,
        y_end,
        z_start,
        z_end,
    };

    VtkData {
        locations: Locations {
            x_locations,
            y_locations,
            z_locations,
        },
        data: D::from(data),
        spans,
    }
}

#[test]
fn check_combining() {
    fn make_data(i: Vec<usize>) -> crate::helpers::SpanData {
        let mapped: Vec<f64> = i.into_iter().map(|x| x as f64).collect();
        crate::helpers::SpanData {
            rho: mapped.clone(),
        }
    }

    let vtk_0 = VtkData {
        data: make_data(vec![1, 2, 3, 7, 8, 9]),
        locations: Locations {
            x_locations: vec![1., 2., 3.],
            y_locations: vec![1., 2.],
            z_locations: vec![0.],
        },
        spans: LocationSpans {
            x_start: 1,
            x_end: 3,
            y_start: 1,
            y_end: 2,
            z_start: 1,
            z_end: 1,
        },
    };

    let proc_0 = crate::helpers::DataItem {
        data: vtk_0.clone(),
        proc_number: 0,
        step_number: 1,
    };

    let vtk_1 = VtkData {
        data: make_data(vec![4, 5, 6, 10, 11, 12]),
        locations: Locations {
            x_locations: vec![4., 5., 6.],
            y_locations: vec![1., 2.],
            z_locations: vec![0.],
        },
        spans: LocationSpans {
            x_start: 4,
            x_end: 6,
            y_start: 1,
            y_end: 2,
            z_start: 1,
            z_end: 1,
        },
    };
    let proc_1 = crate::helpers::DataItem {
        data: vtk_1.clone(),
        proc_number: 1,
        step_number: 1,
    };

    let out_vtk: VtkData<crate::helpers::SpanData> = combine_vtk(vec![proc_0, proc_1]);

    let expected = VtkData {
        locations: Locations {
            x_locations: vec![1., 2., 3., 4., 5., 6.],
            y_locations: vec![1., 2.],
            z_locations: vec![0.],
        },
        data: make_data(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]),
        spans: LocationSpans {
            x_start: 1,
            x_end: 6,
            y_start: 1,
            y_end: 2,
            z_start: 1,
            z_end: 1,
        },
    };

    dbg!(&out_vtk);
    assert_eq!(out_vtk.data, expected.data);
    assert_eq!(out_vtk.locations, expected.locations);
    assert_eq!(out_vtk.spans, expected.spans);
}
