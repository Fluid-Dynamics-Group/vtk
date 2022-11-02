use super::Components;
use crate::prelude::*;

#[derive(Deref, DerefMut, Into, Clone, PartialEq, Default, Debug)]
/// Array container for scalar information in a 3D domain such as pressure
///
/// The first axis should contain X information, and the second axis should contain Y information.
/// No vector information can be stored in `Scalar3D`. If you need to store vector data, see
/// [Field3D](crate::Field3D)
///
/// ## Example
///
/// For some scalar data (such as a pressure field, or density field), if your data is `nx=100` and
/// `ny=200`, `nz=300` then your array shape should be `(100, 200, 300)`
pub struct Scalar3D<NUM>(Array3<NUM>);

impl<NUM> Scalar3D<NUM>
where
    NUM: Numeric,
{
    /// Construct a `Scalar3D` from an array.
    pub fn new(arr: Array3<NUM>) -> Self {
        Self(arr)
    }

    /// get the array that this type wraps.
    /// usually this method is not required because `Scalar3D` implements [`DerefMut`](std::ops::DerefMut) and
    /// [`Deref`](std::ops::Deref)
    pub fn inner(self) -> Array3<NUM> {
        self.0
    }
}

impl FromBuffer<crate::Spans3D> for Scalar3D<f64> {
    fn from_buffer(buffer: Vec<f64>, spans: &crate::Spans3D, components: usize) -> Self {
        let mut arr = Array4::from_shape_vec(
            (components, spans.x_len(), spans.y_len(), spans.z_len()),
            buffer,
        )
        .unwrap();
        // this axes swap accounts for how the data is read. It shoud now match _exactly_
        // how the information is input

        arr.swap_axes(0, 3);
        arr.swap_axes(1, 2);

        Scalar3D::new(
            arr.into_shape((spans.x_len(), spans.y_len(), spans.z_len()))
                .unwrap(),
        )
    }
}

#[derive(Deref)]
pub struct Scalar3DIter<NUM> {
    #[deref]
    arr: Array3<NUM>,
    x: usize,
    y: usize,
    z: usize,
}

impl<NUM> Scalar3DIter<NUM> {
    fn new(arr: Array3<NUM>) -> Self {
        Self {
            arr,
            x: 0,
            y: 0,
            z: 0,
        }
    }
}

impl<NUM> Iterator for Scalar3DIter<NUM>
where
    NUM: Clone + Copy,
{
    type Item = NUM;

    fn next(&mut self) -> Option<Self::Item> {
        let (nz, ny, nx) = self.dim();

        if self.z == nz {
            return None;
        }

        let indexing = (self.z, self.y, self.x);

        // indexing if we are in debug mode
        #[cfg(debug_assertions)]
        let value = *self.arr.get(indexing).unwrap();

        // indexing if we are in release mode
        #[cfg(not(debug_assertions))]
        let value = *unsafe { self.arr.uget(indexing) };

        self.x += 1;

        if self.x == nx {
            self.x = 0;
            self.y += 1;
        }

        if self.y == ny {
            self.y = 0;
            self.z += 1;
        }

        Some(value)
    }
}

impl<NUM> Components for Scalar3D<NUM>
where
    NUM: Clone + num_traits::Zero,
{
    type Iter = Scalar3DIter<NUM>;

    fn array_components(&self) -> usize {
        1
    }

    fn length(&self) -> usize {
        self.len()
    }

    fn iter(&self) -> Self::Iter {
        let mut arr = ndarray::Array::zeros(self.0.t().dim());
        arr.assign(&self.0.t());
        Scalar3DIter::new(arr)
    }
}

#[test]
fn iter_order() {
    let nx = 3;
    let ny = 2;
    let nz = 4;

    let arr = ndarray::Array1::range(0., (nx * ny * nz) as f64, 1.)
        .into_shape((nx, ny, nz))
        .unwrap();
    dbg!(&arr);
    let mut expected = Vec::new();

    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                println!("GOAL INDEXING AT {} {}", i, j);
                expected.push(*arr.get((i, j, k)).unwrap());
            }
        }
    }

    let actual = Scalar3D(arr).iter().collect::<Vec<_>>();

    assert_eq!(expected, actual)
}
