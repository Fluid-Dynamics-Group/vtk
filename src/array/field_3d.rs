use super::Components;
use crate::prelude::*;

#[derive(Constructor, Deref, DerefMut, Into, Clone, PartialEq, Default, Debug)]
pub struct Field3D(Array4<f64>);

#[derive(Deref)]
pub struct Field3DIter {
    #[deref]
    arr: Array4<f64>,
    n: usize,
    x: usize,
    y: usize,
    z: usize,
}

impl FromBuffer<crate::Spans3D> for Field3D {
    fn from_buffer(buffer: Vec<f64>, spans: &crate::Spans3D, components: usize) -> Self {
        let mut arr = Array4::from_shape_vec(
            (components, spans.x_len(), spans.y_len(), spans.z_len()),
            buffer,
        )
        .unwrap();
        // this axes swap accounts for how the data is read. It shoud now match _exactly_
        // how the information is input
        
        arr.swap_axes(0,3);
        arr.swap_axes(1,2);

        Field3D::new(arr)
    }
}

impl Field3DIter {
    fn new(arr: Array4<f64>) -> Self {
        Self {
            arr,
            x: 0,
            y: 0,
            z: 0,
            n: 0,
        }
    }
}

impl Iterator for Field3DIter {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        let (nn, nx, ny, nz) = self.dim();

        if self.z == nz {
            return None;
        }

        let value = *self.arr.get((self.n, self.x, self.y, self.z)).unwrap();

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

impl Components for Field3D {
    type Iter = Field3DIter;

    fn array_components(&self) -> usize {
        self.dim().0
    }

    fn length(&self) -> usize {
        self.len()
    }

    fn iter(&self) -> Self::Iter {
        Field3DIter::new(self.0.clone())
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
