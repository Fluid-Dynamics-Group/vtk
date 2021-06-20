use super::iter::VtkIterator;
use std::ops::{Add, Div, Sub, SubAssign};

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

#[derive(Debug, Clone, Default, derive_builder::Builder, PartialEq)]
pub struct Locations {
    pub x_locations: Vec<f64>,
    pub y_locations: Vec<f64>,
    pub z_locations: Vec<f64>,
}

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

    pub fn x_len(&self) -> usize {
        self.x_end - self.x_start + 1
    }

    pub fn y_len(&self) -> usize {
        self.y_end - self.y_start + 1
    }

    pub fn z_len(&self) -> usize {
        self.z_end - self.z_start + 1
    }

    pub fn to_string(&self) -> String {
        format!(
            "{} {} {} {} {} {}",
            self.x_start, self.x_end, self.y_start, self.y_end, self.z_start, self.z_end
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_add() {
        let data = SpanData {
            rho: vec![0., 1., 2.],
            u: vec![0., 1., 2.],
            v: vec![0., 1., 2.],
            w: vec![0., 1., 2.],
            energy: vec![0., 1., 2.],
        };
        let data_2 = SpanData {
            rho: vec![0., 0., 1.],
            u: vec![0., 0., 1.],
            v: vec![0., 0., 1.],
            w: vec![0., 0., 1.],
            energy: vec![0., 0., 1.],
        };
        let expected = SpanData {
            rho: vec![0., 1., 3.],
            u: vec![0., 1., 3.],
            v: vec![0., 1., 3.],
            w: vec![0., 1., 3.],
            energy: vec![0., 1., 3.],
        };

        assert_eq!(data + data_2, expected)
    }

    #[test]
    fn data_div() {
        let data = SpanData {
            rho: vec![3., 3., 3.],
            u: vec![3., 3., 3.],
            v: vec![3., 3., 3.],
            w: vec![3., 3., 3.],
            energy: vec![3., 3., 3.],
        };
        let expected = SpanData {
            rho: vec![1., 1., 1.],
            u: vec![1., 1., 1.],
            v: vec![1., 1., 1.],
            w: vec![1., 1., 1.],
            energy: vec![1., 1., 1.],
        };

        assert_eq!(data / 3., expected)
    }
}
