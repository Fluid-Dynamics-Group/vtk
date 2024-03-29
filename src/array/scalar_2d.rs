use super::Components;
use crate::prelude::*;

#[derive(Deref, DerefMut, Into, Clone, PartialEq, Default, Debug)]
/// Array data container for scalar information in a 2D domain such as pressure
///
/// The first axis should contain X information, and the second axis should contain Y information.
/// No vector information can be stored in `Scalar2D`. If you need to store vector data, see
/// [Vector2D](crate::Vector2D)
///
/// ## Example
///
/// For some scalar data (such as a pressure field, or density field), if your data is `nx=100` and
/// `ny=200` then your array shape should be `(100, 200)`
pub struct Scalar2D<NUM>(Array2<NUM>);

impl<NUM> Scalar2D<NUM>
where
    NUM: Numeric,
{
    /// Construct a `Scalar2D` from an array.
    pub fn new(arr: Array2<NUM>) -> Self {
        Self(arr)
    }

    /// get the array that this type wraps.
    /// usually this method is not required because `Scalar2D` implements [`DerefMut`](std::ops::DerefMut) and
    /// [`Deref`](std::ops::Deref)
    pub fn inner(self) -> Array2<NUM> {
        self.0
    }
}

impl FromBuffer<crate::Spans2D> for Scalar2D<f64> {
    fn from_buffer(buffer: Vec<f64>, spans: &crate::Spans2D, _: usize) -> Self {
        let mut arr = Array4::from_shape_vec((spans.x_len(), spans.y_len(), 1, 1), buffer).unwrap();

        // this axes swap accounts for how the data is read. It shoud now match _exactly_
        // how the information is input

        arr.swap_axes(0, 3);
        arr.swap_axes(1, 2);

        Scalar2D::new(arr.into_shape((spans.x_len(), spans.y_len())).unwrap())
    }
}

#[derive(Deref)]
pub struct Scalar2DIter<NUM> {
    #[deref]
    arr: Array2<NUM>,
    x: usize,
    y: usize,
}

impl<NUM> Scalar2DIter<NUM> {
    fn new(arr: Array2<NUM>) -> Self {
        Self { arr, x: 0, y: 0 }
    }
}

impl<NUM> Iterator for Scalar2DIter<NUM>
where
    NUM: Clone + Copy,
{
    type Item = NUM;

    fn next(&mut self) -> Option<Self::Item> {
        let (ny, nx) = self.dim();

        if self.y == ny {
            return None;
        }

        let indexing = (self.y, self.x);

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

        Some(value)
    }
}

impl<NUM> Components for Scalar2D<NUM>
where
    NUM: Clone + num_traits::Zero,
{
    type Iter = Scalar2DIter<NUM>;

    fn array_components(&self) -> usize {
        1
    }

    fn length(&self) -> usize {
        self.len()
    }

    fn iter(&self) -> Scalar2DIter<NUM> {
        let mut arr = ndarray::Array::zeros(self.0.t().dim());
        arr.assign(&self.0.t());
        Scalar2DIter::new(arr)
    }
}

#[test]
fn iter_order() {
    let arr = ndarray::arr2(&[[1., 2.], [3., 4.]]);
    let mut expected = Vec::new();

    for j in 0..2 {
        for i in 0..2 {
            println!("GOAL INDEXING AT {} {}", i, j);
            expected.push(*arr.get((i, j)).unwrap());
        }
    }

    let actual = Scalar2D(arr).iter().collect::<Vec<_>>();

    assert_eq!(expected, actual)
}
