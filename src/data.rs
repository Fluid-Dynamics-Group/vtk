use super::iter::VtkIterator;
use std::ops::{Add, Div, Sub, SubAssign};

/// Stores vector information at each point in space instead
/// of a single scalar value #[derive(Clone, Debug)]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct VectorPoints {
    pub(crate) components: usize,
    pub(crate) arr: ndarray::Array4<f64>,
}

impl VectorPoints {
    /// The passed in array should be in real space, not
    /// in the vtk-space
    ///
    /// Vtk-space writing will be taken care of
    pub fn new(arr: ndarray::Array4<f64>) -> Self {
        let components = arr.raw_dim()[3];

        Self { components, arr }
    }

    pub fn dims(&self) -> (usize, usize, usize) {
        let dims = self.arr.raw_dim();
        let nx = dims[0];
        let ny = dims[1];
        let nz = dims[2];

        (nx, ny, nz)
    }
}

impl std::ops::Deref for VectorPoints {
    type Target = ndarray::Array4<f64>;

    fn deref(&self) -> &Self::Target {
        &self.arr
    }
}

impl std::ops::DerefMut for VectorPoints {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.arr
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct VtkData<D> {
    pub data: D,
    pub locations: Locations,
    pub spans: LocationSpans,
}

impl <D> VtkData <D> {
    pub fn new_data<T>(self, new_data: T) -> VtkData<T> {
        let spans = self.spans;
        let locations = self.locations;
        
        VtkData { spans, locations, data : new_data }
    }
}

impl<D> Add for VtkData<D>
where
    D: Add<Output = D>,
{
    type Output = Self;

    fn add(mut self, other: Self) -> Self::Output {
        self.data = self.data + other.data;
        self
    }
}

impl<D> Div<f64> for VtkData<D>
where
    D: Div<f64, Output = D>,
{
    type Output = Self;

    fn div(mut self, other: f64) -> Self::Output {
        self.data = self.data / other;
        self
    }
}

impl<D> Sub for VtkData<D>
where
    D: Sub<Output = D>,
{
    type Output = Self;

    fn sub(mut self, other: Self) -> Self::Output {
        self.data = self.data - other.data;
        self
    }
}

impl<D> SubAssign for VtkData<D>
where
    D: Sub<Output = D> + Clone,
{
    fn sub_assign(&mut self, other: Self) {
        self.data = self.data.clone() - other.data;
    }
}

impl<D> std::iter::Sum for VtkData<D>
where
    D: Default,
    VtkData<D>: Add<Output = Self>,
{
    fn sum<I: Iterator<Item = VtkData<D>>>(iter: I) -> Self {
        let mut base = VtkData::default();
        for i in iter {
            // important to write it this way so that base will
            // be overwritten by the new spans of the iterator
            // since the spans on the default are incorrect
            base = i + base;
        }
        base
    }
}

impl<D> std::iter::IntoIterator for VtkData<D>
where
    D: super::traits::PointData,
{
    type Item = D::PointData;
    type IntoIter = VtkIterator<D>;

    fn into_iter(self) -> Self::IntoIter {
        VtkIterator::new(self.data)
    }
}

/// The X/Y/Z point data locations for the data points in the field
#[derive(Debug, Clone, Default, derive_builder::Builder, PartialEq)]
pub struct Locations {
    pub x_locations: Vec<f64>,
    pub y_locations: Vec<f64>,
    pub z_locations: Vec<f64>,
}

/// The local locations
#[derive(Debug, Clone, Default, PartialEq)]
pub struct LocationSpans {
    pub x_start: usize,
    pub x_end: usize,
    pub y_start: usize,
    pub y_end: usize,
    pub z_start: usize,
    pub z_end: usize,
}

impl LocationSpans {
    /// simple constructor used to generate a `LocationSpans` from a string
    /// you would find in a vtk file. The expeceted input is in the form
    /// `"x_start x_end y_start y_end z_start z_end"`
    ///
    /// # Example
    /// ```
    /// vtk::LocationSpans::new("0 10 0 20 0 10");
    /// ```
    ///
    /// ## Panics
    ///
    /// This function panics if there are not 6 `usize` values
    /// separated by a single space each
    pub fn new(span_string: &str) -> Self {
        let mut split = span_string.split_ascii_whitespace();

        LocationSpans {
            x_start: split.next().unwrap().parse().unwrap(),
            x_end: split.next().unwrap().parse().unwrap(),
            y_start: split.next().unwrap().parse().unwrap(),
            y_end: split.next().unwrap().parse().unwrap(),
            z_start: split.next().unwrap().parse().unwrap(),
            z_end: split.next().unwrap().parse().unwrap(),
        }
    }

    /// Get the total length in the X direction for this
    /// local segment as paraview would interpret it
    pub fn x_len(&self) -> usize {
        self.x_end - self.x_start + 1
    }

    /// Get the total length in the Y direction for this
    /// local segment as paraview would interpret it
    pub fn y_len(&self) -> usize {
        self.y_end - self.y_start + 1
    }

    /// Get the total length in the Z direction for this
    /// local segment as paraview would interpret it
    pub fn z_len(&self) -> usize {
        self.z_end - self.z_start + 1
    }

    /// Format the spans into a string that would be written to a vtk file
    pub(crate) fn to_string(&self) -> String {
        format!(
            "{} {} {} {} {} {}",
            self.x_start, self.x_end, self.y_start, self.y_end, self.z_start, self.z_end
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::helpers::SpanData;
    use crate::VectorPoints;

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
        array: crate::VectorPoints,
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
        let arr = arr.reversed_axes();

        dbg!(arr[[0, 0, 0, 0]], arr[[0, 0, 0, 1]], arr[[0, 0, 0, 2]],);

        let data = SimpleArray {
            array: VectorPoints::new(arr),
        };

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
        panic!()
    }

    #[test]
    fn read_simple_vtk_after_write() {
        let mut file = Vec::new();
        let vtk = setup_vtk();
        let data = vtk.data.clone();
        vtk::write_vtk(&mut file, vtk, true).unwrap();

        let out_vtk = crate::parse::parse_xml_document::<SimpleArray>(&file).unwrap();
        let out_data = out_vtk.data;

        assert_eq!(data.array.arr, out_data.array.arr);
    }
}
