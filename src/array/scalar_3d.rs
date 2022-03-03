use super::Components;
use crate::prelude::*;

#[derive(Constructor, Deref, DerefMut, Into, Clone, PartialEq, Default, Debug)]
/// Array container for scalar information in a 3D domain such as pressure
pub struct Scalar3D(Array3<f64>);

impl FromBuffer<crate::Spans3D> for Scalar3D {
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
pub struct Scalar3DIter {
    #[deref]
    arr: Array3<f64>,
    x: usize,
    y: usize,
    z: usize,
}

impl Scalar3DIter {
    fn new(arr: Array3<f64>) -> Self {
        Self {
            arr,
            x: 0,
            y: 0,
            z: 0,
        }
    }
}

impl Iterator for Scalar3DIter {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        let (nx, ny, nz) = self.dim();

        if self.z == nz {
            return None;
        }

        let value = *self.arr.get((self.x, self.y, self.z)).unwrap();

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

impl Components for Scalar3D {
    type Iter = Scalar3DIter;

    fn array_components(&self) -> usize {
        1
    }

    fn length(&self) -> usize {
        self.len()
    }

    fn iter(&self) -> Self::Iter {
        Scalar3DIter::new(self.0.clone())
    }
}

#[test]
fn iter_order() {
    let nx = 3;
    let ny = 3;
    let nz = 3;

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
