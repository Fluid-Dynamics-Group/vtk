use super::iter::VtkIterator;
use super::Data;
use crate::Arr2;
use std::io::Write;
use std::ops::{Add, Div, Sub, SubAssign};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct VtkData<D: Data> {
    pub data: D,
    pub locations: Locations,
    pub spans: LocationSpans,
}

impl VtkData<SpanData> {
    //pub(crate) fn into_array(self) -> (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>) {
    pub(crate) fn extend_all(
        self,
        rho: &mut Vec<f64>,
        u: &mut Vec<f64>,
        v: &mut Vec<f64>,
        w: &mut Vec<f64>,
        energy: &mut Vec<f64>,
    ) {
        rho.extend(self.data.rho);
        u.extend(self.data.u);
        v.extend(self.data.v);
        w.extend(self.data.w);
        energy.extend(self.data.energy);
    }
}

impl<D> Add for VtkData<D>
where
    D: Add<Output = D> + Data,
{
    type Output = Self;

    fn add(mut self, other: Self) -> Self::Output {
        self.data = self.data + other.data;
        self
    }
}

impl<D> Div<f64> for VtkData<D>
where
    D: Div<f64, Output = D> + Data,
{
    type Output = Self;

    fn div(mut self, other: f64) -> Self::Output {
        self.data = self.data / other;
        self
    }
}

impl<D> Sub for VtkData<D>
where
    D: Sub<Output = D> + Data,
{
    type Output = Self;

    fn sub(mut self, other: Self) -> Self::Output {
        self.data = self.data - other.data;
        self
    }
}

impl<D> SubAssign for VtkData<D>
where
    D: Data + Sub<Output = D>,
{
    fn sub_assign(&mut self, other: Self) {
        self.data = self.data.clone() - other.data;
    }
}

// TODO: make this function + iterator generic at some point
impl std::iter::Sum for VtkData<SpanData> {
    fn sum<I: Iterator<Item = VtkData<SpanData>>>(iter: I) -> Self {
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

impl std::iter::IntoIterator for VtkData<SpanData> {
    type Item = super::iter::PointData;
    type IntoIter = VtkIterator;

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

#[derive(Debug, Clone, Default, derive_builder::Builder, PartialEq)]
pub struct SpanData {
    pub rho: Vec<f64>,
    pub u: Vec<f64>,
    pub v: Vec<f64>,
    pub w: Vec<f64>,
    pub energy: Vec<f64>,
}

impl Data for SpanData {}

impl super::DataArray for SpanData {
    fn write_dataarray<W: Write>(
        self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::Error> {
        super::write_vtk::write_dataarray(writer, self.rho, "rho", false)?;
        super::write_vtk::write_dataarray(writer, self.u, "u", false)?;
        super::write_vtk::write_dataarray(writer, self.v, "v", false)?;
        super::write_vtk::write_dataarray(writer, self.w, "w", false)?;
        super::write_vtk::write_dataarray(writer, self.energy, "energy", false)?;
        Ok(())
    }
}

impl Add for SpanData {
    type Output = Self;

    fn add(mut self, other: Self) -> Self {
        self.rho
            .iter_mut()
            .zip(other.rho.into_iter())
            .for_each(|(s, o)| *s = *s + o);
        self.u
            .iter_mut()
            .zip(other.u.into_iter())
            .for_each(|(s, o)| *s = *s + o);
        self.v
            .iter_mut()
            .zip(other.v.into_iter())
            .for_each(|(s, o)| *s = *s + o);
        self.w
            .iter_mut()
            .zip(other.w.into_iter())
            .for_each(|(s, o)| *s = *s + o);
        self.energy
            .iter_mut()
            .zip(other.energy.into_iter())
            .for_each(|(s, o)| *s = *s + o);
        self
    }
}

impl Div<f64> for SpanData {
    type Output = Self;

    fn div(mut self, other: f64) -> Self::Output {
        self.rho.iter_mut().for_each(|s| *s = *s / other);
        self.u.iter_mut().for_each(|s| *s = *s / other);
        self.v.iter_mut().for_each(|s| *s = *s / other);
        self.w.iter_mut().for_each(|s| *s = *s / other);
        self.energy.iter_mut().for_each(|s| *s = *s / other);
        self
    }
}

impl Sub for SpanData {
    type Output = Self;

    fn sub(mut self, other: Self) -> Self {
        self.rho
            .iter_mut()
            .zip(other.rho.into_iter())
            .for_each(|(s, o)| *s = *s - o);
        self.u
            .iter_mut()
            .zip(other.u.into_iter())
            .for_each(|(s, o)| *s = *s - o);
        self.v
            .iter_mut()
            .zip(other.v.into_iter())
            .for_each(|(s, o)| *s = *s - o);
        self.w
            .iter_mut()
            .zip(other.w.into_iter())
            .for_each(|(s, o)| *s = *s - o);
        self.energy
            .iter_mut()
            .zip(other.energy.into_iter())
            .for_each(|(s, o)| *s = *s - o);
        self
    }
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
    pub(crate) fn new(span_string: &str) -> Self {
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

    pub(crate) fn x_len(&self) -> usize {
        self.x_end - self.x_start + 1
    }

    pub(crate) fn y_len(&self) -> usize {
        self.y_end - self.y_start + 1
    }

    pub(crate) fn z_len(&self) -> usize {
        self.z_end - self.z_start + 1
    }

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
