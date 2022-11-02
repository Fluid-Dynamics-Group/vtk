use super::Components;
use crate::prelude::*;

#[derive(Deref, DerefMut, Into, Clone, PartialEq, Default, Debug)]
/// Array container for vector information in a 2D domain such as velocity
///
/// The first axis should contain the vector information, and the second / third axis should
/// contain X / Y information
///
/// ## Example
///
/// For velocity, in a domain `nx = 100` and `ny = 200`, the array needs to have
/// the shape `(3, 100, 200)`
pub struct Field2D<NUM>(Array3<NUM>);

impl<NUM> Field2D<NUM>
where
    NUM: Numeric,
{
    /// Construct a `Field2D` from an array.
    pub fn new(arr: Array3<NUM>) -> Self {
        Self(arr)
    }

    /// get the array that this type wraps.
    /// usually this method is not required because `Field2D` implements [`DerefMut`](std::ops::DerefMut) and
    /// [`Deref`](std::ops::Deref)
    pub fn inner(self) -> Array3<NUM> {
        self.0
    }
}

impl FromBuffer<crate::Spans2D> for Field2D<f64> {
    fn from_buffer(buffer: Vec<f64>, spans: &crate::Spans2D, components: usize) -> Self {
        let mut arr =
            Array4::from_shape_vec((components, spans.x_len(), spans.y_len(), 1), buffer).unwrap();

        arr.swap_axes(0, 3);
        arr.swap_axes(1, 2);

        Field2D::new(
            arr.into_shape((components, spans.x_len(), spans.y_len()))
                .unwrap(),
        )
    }
}

#[derive(Deref)]
pub struct Field2DIter<NUM> {
    #[deref]
    arr: Array3<NUM>,
    x: usize,
    y: usize,
    n: usize,
}

impl<NUM> Field2DIter<NUM> {
    fn new(arr: Array3<NUM>) -> Self {
        Self {
            arr,
            x: 0,
            y: 0,
            n: 0,
        }
    }
}

impl<NUM> Iterator for Field2DIter<NUM>
where
    NUM: Copy + Clone,
{
    type Item = NUM;

    fn next(&mut self) -> Option<Self::Item> {
        let (ny, nx, nn) = self.dim();

        if self.y == ny {
            return None;
        }

        let indexing = (self.y, self.x, self.n);

        // debug mode code
        #[cfg(debug_assertions)]
        let value = *self.arr.get(indexing).unwrap();

        // release mode code
        #[cfg(not(debug_assertions))]
        let value = *unsafe { self.arr.uget(indexing) };

        self.n += 1;

        // inner most loop
        if self.n == nn {
            self.n = 0;
            self.x += 1;
        }

        // second inner most loop
        if self.x == nx {
            self.x = 0;
            self.y += 1;
        }

        Some(value)
    }
}

impl<NUM> Components for Field2D<NUM>
where
    NUM: Copy + Clone + num_traits::Zero,
{
    type Iter = Field2DIter<NUM>;

    fn array_components(&self) -> usize {
        self.dim().0
    }

    fn length(&self) -> usize {
        self.len()
    }

    fn iter(&self) -> Self::Iter {
        let mut arr = ndarray::Array::zeros(self.0.t().dim());
        arr.assign(&self.0.t());
        Field2DIter::new(arr)
    }
}

#[test]
fn iter_order() {
    let nx = 4;
    let ny = 4;
    let nn = 3;

    let arr: Array3<f64> = ndarray::Array1::linspace(0., (nx * ny * nn) as f64 - 1., nx * ny * nn)
        .into_shape((nn, nx, ny))
        .unwrap();

    dbg!(&arr.shape());
    dbg!(&arr);

    let mut expected = Vec::new();

    for j in 0..ny {
        for i in 0..nx {
            for n in 0..nn {
                let val = *arr.get((n, i, j)).unwrap();
                println!("GOAL at ({}, {} @ {}) = {val} ", i, j, n);
                expected.push(val);
            }
        }
    }

    println!("on current , velocity at (2,3) is");
    let cv = &arr;
    println!("({}, {}, {})", cv[(0, 2, 3)], cv[(1, 2, 3)], cv[(2, 2, 3)]);

    println!("using the field construct, velocity at (2,3) is");
    let cv = Field2D::new(arr.clone());
    println!("({}, {}, {})", cv[(0, 2, 3)], cv[(1, 2, 3)], cv[(2, 2, 3)]);

    let actual = Field2D::new(arr).iter().collect::<Vec<_>>();

    assert_eq!(expected, actual);
}
