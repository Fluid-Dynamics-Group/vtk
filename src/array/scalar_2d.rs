use super::Components;
use crate::prelude::*;

#[derive(Constructor, Deref, DerefMut, Into, Clone, PartialEq, Default, Debug)]
/// Array data container for scalar information in a 2D domain such as pressure
pub struct Scalar2D(Array2<f64>);

impl FromBuffer<crate::Spans2D> for Scalar2D {
    fn from_buffer(buffer: Vec<f64>, spans: &crate::Spans2D, _: usize) -> Self {
        println!(
            "buffer length {} x * y {} x {} y {}",
            buffer.len(),
            spans.x_len() * spans.y_len(),
            spans.x_len(),
            spans.y_len()
        );
        println!("calling from_buffer for scalar2d");

        let mut arr = Array4::from_shape_vec((spans.x_len(), spans.y_len(), 1, 1), buffer).unwrap();

        // this axes swap accounts for how the data is read. It shoud now match _exactly_
        // how the information is input

        arr.swap_axes(0, 3);
        arr.swap_axes(1, 2);

        Scalar2D::new(arr.into_shape((spans.x_len(), spans.y_len())).unwrap())
    }
}

#[derive(Deref)]
pub struct Scalar2DIter {
    #[deref]
    arr: Array2<f64>,
    x: usize,
    y: usize,
}

impl Scalar2DIter {
    fn new(arr: Array2<f64>) -> Self {
        Self { arr, x: 0, y: 0 }
    }
}

impl Iterator for Scalar2DIter {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        let (nx, ny) = self.dim();

        if self.y == ny {
            return None;
        }

        let value = *self.arr.get((self.x, self.y)).unwrap();

        self.x += 1;

        if self.x == nx {
            self.x = 0;
            self.y += 1;
        }

        Some(value)
    }
}

impl Components for Scalar2D {
    type Iter = Scalar2DIter;

    fn array_components(&self) -> usize {
        1
    }

    fn length(&self) -> usize {
        self.len()
    }

    fn iter(&self) -> Scalar2DIter {
        Scalar2DIter::new(self.0.clone())
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
