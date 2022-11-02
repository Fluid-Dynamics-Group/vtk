use super::Components;
use crate::prelude::*;

#[derive(Deref, DerefMut, Into, Clone, PartialEq, Default, Debug)]
/// Array container for vector information in a 3D domain such as velocity
///
/// The first axis should contain the vector information, and the second / third / fourth axis should
/// contain X / Y / Z information
///
/// ## Example
///
/// For velocity, in a domain `nx = 100` and `ny = 200`, `nz=300`, the array needs to have
/// the shape `(3, 100, 200, 300)`
pub struct Field3D<NUM>(Array4<NUM>);

impl<NUM> Field3D<NUM>
where
    NUM: Numeric,
{
    /// Construct a `Field3D` from an array.
    pub fn new(arr: Array4<NUM>) -> Self {
        Self(arr)
    }

    /// get the array that this type wraps.
    /// usually this method is not required because `Field3D` implements [`DerefMut`](std::ops::DerefMut) and
    /// [`Deref`](std::ops::Deref)
    pub fn inner(self) -> Array4<NUM> {
        self.0
    }
}

#[derive(Deref)]
pub struct Field3DIter<NUM> {
    #[deref]
    pub arr: Array4<NUM>,
    n: usize,
    x: usize,
    y: usize,
    z: usize,
}

impl FromBuffer<crate::Spans3D> for Field3D<f64> {
    fn from_buffer(buffer: Vec<f64>, spans: &crate::Spans3D, components: usize) -> Self {
        let mut arr = ndarray::Array5::from_shape_vec(
            (components, spans.x_len(), spans.y_len(), spans.z_len(), 1),
            buffer,
        )
        .unwrap();
        // this axes swap accounts for how the data is read. It shoud now match _exactly_
        // how the information is input

        arr.swap_axes(0, 3);
        arr.swap_axes(1, 2);

        let arr = arr
            .into_shape((components, spans.x_len(), spans.y_len(), spans.z_len()))
            .unwrap();
        Field3D::new(arr)
    }
}

impl<NUM> Field3DIter<NUM> {
    fn new(arr: Array4<NUM>) -> Self {
        Self {
            arr,
            x: 0,
            y: 0,
            z: 0,
            n: 0,
        }
    }
}

impl<NUM> Iterator for Field3DIter<NUM>
where
    NUM: Clone + Copy,
{
    type Item = NUM;

    fn next(&mut self) -> Option<Self::Item> {
        let (nz, ny, nx, nn) = self.dim();

        if self.z == nz {
            return None;
        }

        let indexing = (self.z, self.y, self.x, self.n);

        // indexing if we are in debug mode
        #[cfg(debug_assertions)]
        let value = *self.arr.get(indexing).unwrap();

        // indexing if we are in release mode
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

        // third most inner loop
        if self.y == ny {
            self.y = 0;
            self.z += 1;
        }

        Some(value)
    }
}

impl<NUM> Components for Field3D<NUM>
where
    NUM: Clone + num_traits::Zero,
{
    type Iter = Field3DIter<NUM>;

    fn array_components(&self) -> usize {
        self.dim().0
    }

    fn length(&self) -> usize {
        self.len()
    }

    fn iter(&self) -> Self::Iter {
        let mut arr = ndarray::Array::zeros(self.0.t().dim());
        arr.assign(&self.0.t());
        Field3DIter::new(arr)
    }
}

#[test]
fn iter_order() {
    let nx = 3;
    let ny = 3;
    let nz = 3;
    let nn = 2;

    let arr: Array4<f64> = ndarray::Array1::range(0., (nx * ny * nz * nn) as f64, 1.)
        .into_shape((nn, nx, ny, nz))
        .unwrap();

    dbg!(&arr);
    let mut expected = Vec::new();

    for k in 0..ny {
        for j in 0..ny {
            for i in 0..nx {
                for n in 0..nn {
                    println!("GOAL INDEXING AT {} {}", i, j);
                    expected.push(*arr.get((n, i, j, k)).unwrap());
                }
            }
        }
    }

    let actual = Field3D::new(arr).iter().collect::<Vec<_>>();

    assert_eq!(expected, actual)
}
