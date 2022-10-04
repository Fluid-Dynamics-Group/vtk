use super::Components;
use crate::prelude::*;

#[derive(Constructor, Deref, DerefMut, Into, Clone, PartialEq, Default, Debug)]
/// Array container for scalar information in a 2D domain such as velocity
pub struct Field2D(Array3<f64>);

impl FromBuffer<crate::Spans2D> for Field2D {
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
pub struct Field2DIter {
    #[deref]
    arr: Array3<f64>,
    x: usize,
    y: usize,
    n: usize,
}

impl Field2DIter {
    fn new(arr: Array3<f64>) -> Self {
        Self {
            arr,
            x: 0,
            y: 0,
            n: 0,
        }
    }
}

impl Iterator for Field2DIter {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        let (nn, nx, ny) = self.dim();

        if self.y == ny {
            return None;
        }

        let value = *self.arr.get((self.n, self.x, self.y)).unwrap();

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

impl Components for Field2D {
    type Iter = Field2DIter;

    fn array_components(&self) -> usize {
        self.dim().0
    }

    fn length(&self) -> usize {
        self.len()
    }

    fn iter(&self) -> Self::Iter {
        Field2DIter::new(self.0.clone())
    }
}

#[test]
fn iter_order() {
    let nx = 4;
    let ny = 4;
    let nn = 3;

    let arr: Array3<f64> = ndarray::Array1::linspace(0., (nx * ny * nn) as f64-1., nx* ny * nn)
        .into_shape((nn, nx, ny))
        .unwrap();

    dbg!(&arr.shape());
    dbg!(&arr);

    let mut expected = Vec::new();

    for j in 0..ny {
        for i in 0..nx {
            for n in 0..nn {
                let val = *arr.get((n, i, j)).unwrap();
                println!("GOAL at ({}, {} @ {}) = {val} ",  i, j, n);
                expected.push(val);
            }
        }
    }

    println!("on current , velocity at (2,3) is");
    let cv = &arr;
    println!("({}, {}, {})", cv[(0, 2,3)], cv[(1,2,3)], cv[(2,2,3)]);

    println!("using the field construct, velocity at (2,3) is");
    let cv = Field2D::new(arr.clone());
    println!("({}, {}, {})", cv[(0, 2,3)], cv[(1,2,3)], cv[(2,2,3)]);

    let actual = Field2D::new(arr).iter().collect::<Vec<_>>();

    assert_eq!(expected, actual);
}
