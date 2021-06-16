use super::data::{LocationSpans, Locations, SpanData, VtkData};
use crate::data::DataItem;

fn allocate_data<T>(span_info: &LocationSpans) -> Vec<T> {
    Vec::with_capacity(span_info.y_len() * span_info.x_len())
}

pub fn combine_vtk(data: Vec<DataItem>) -> VtkData<SpanData> {
    let total_procs = data.len();

    let span_info = &data[0].data.spans;

    let mut rho = allocate_data(&span_info);
    let mut u = allocate_data(&span_info);
    let mut v = allocate_data(&span_info);
    let mut w = allocate_data(&span_info);
    let mut energy = allocate_data(&span_info);

    let mut x_locations = Vec::with_capacity(total_procs * span_info.x_len());
    let y_locations = data[0].data.locations.y_locations.clone();
    let z_locations = vec![0.];

    for data_item in data.iter() {
        for x in data_item.data.locations.x_locations.iter() {
            x_locations.push(*x)
        }
    }

    for j in 0..span_info.y_len() {
        for data_item in data.iter() {
            for i in 0..span_info.x_len() {
                let index = i + (j * span_info.x_len());

                rho.push(data_item.data.data.rho[index]);
                u.push(data_item.data.data.u[index]);
                v.push(data_item.data.data.v[index]);
                w.push(data_item.data.data.w[index]);
                energy.push(data_item.data.data.energy[index]);
            }
        }
    }

    let spans = LocationSpans {
        x_start: data[0].data.spans.x_start,
        x_end: data[total_procs - 1].data.spans.x_end,
        y_start: span_info.y_start,
        y_end: span_info.y_end,
        z_start: span_info.z_start,
        z_end: span_info.z_end,
    };

    VtkData {
        locations: Locations {
            x_locations,
            y_locations,
            z_locations,
        },
        data: SpanData {
            rho,
            u,
            v,
            w,
            energy,
        },
        spans,
    }
}

#[test]
fn check_combining() {
    fn make_data(i: Vec<usize>) -> SpanData {
        let mapped: Vec<f64> = i.into_iter().map(|x| x as f64).collect();
        SpanData {
            rho: mapped.clone(),
            u: mapped.clone(),
            v: mapped.clone(),
            w: mapped.clone(),
            energy: mapped,
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
    let proc_0 = DataItem {
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
    let proc_1 = DataItem {
        data: vtk_1.clone(),
        proc_number: 0,
        step_number: 1,
    };

    let out_vtk = combine_vtk(vec![proc_0, proc_1]);
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
