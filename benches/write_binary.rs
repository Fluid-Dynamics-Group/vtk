use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ndarray::Array;
use ndarray::Array4;
use ndarray_rand::rand_distr::Uniform;
use ndarray_rand::RandomExt;

use vtk::array::Components;
use vtk::Array as _;

fn write_binary(n: usize) -> () {
    let array: Array4<f32> = ndarray::Array::random((3, n, n, n), Uniform::new(0., 10.));

    let container = vtk::Field3D::new(array);

    let writer: Vec<u8> = Vec::new();
    let buf_writer = std::io::BufWriter::new(writer);
    let mut event_writer = vtk::EventWriter::new(buf_writer);

    container.write_binary(&mut event_writer, false).unwrap();
}

fn write_binary_bench(c: &mut Criterion) {
    c.bench_function("write binary 100", |b| {
        b.iter(|| write_binary(black_box(100)))
    });

    c.bench_function("write binary 150", |b| {
        b.iter(|| write_binary(black_box(150)))
    });
}

criterion_group!(benches, write_binary_bench);
criterion_main!(benches);
