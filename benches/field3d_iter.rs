use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ndarray::Array4;
use ndarray_rand::rand_distr::Uniform;
use ndarray_rand::RandomExt;

use vtk::array::Components;

fn vector_3d_collect(n: usize) -> f32 {
    let array: Array4<f32> = ndarray::Array::random((3, n, n, n), Uniform::new(0., 10.));

    let container = vtk::Vector3D::new(array);
    let iter: vtk::array::Vector3DIter<f32> = container.iter();
    let iter = iter.arr;
    iter.sum()
}

fn vector3d_bench(c: &mut Criterion) {
    c.bench_function("field3d_collect 100", |b| {
        b.iter(|| vector_3d_collect(black_box(100)))
    });

    c.bench_function("field3d_collect 150", |b| {
        b.iter(|| vector_3d_collect(black_box(150)))
    });
}

criterion_group!(benches, vector3d_bench);
criterion_main!(benches);
