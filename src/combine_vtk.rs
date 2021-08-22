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
