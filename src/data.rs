use super::iter::VtkIterator;
use std::ops::{Add, Div, Sub, SubAssign};
use crate::utils;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct VtkData<D> {
    pub data: D,
    pub locations: Locations,
    pub spans: LocationSpans,
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

// TODO: make this function + iterator generic at some point
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

    #[test]
    fn data_add() {
        let data = SpanData {
            rho: vec![0., 1., 2.],
        };
        let data_2 = SpanData {
            rho: vec![0., 0., 1.],
        };
        let expected = SpanData {
            rho: vec![0., 1., 3.],
        };

        assert_eq!(data + data_2, expected)
    }

    #[test]
    fn data_div() {
        let data = SpanData {
            rho: vec![3., 3., 3.],
        };
        let expected = SpanData {
            rho: vec![1., 1., 1.],
        };

        assert_eq!(data / 3., expected)
    }
}
