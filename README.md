# vtk

![](./static/header.png)

 Library for reading and writing vtk files

## Supported Features

* writing XML Rectilinear grids
	* Ascii
	* Binary
	* Base64
* Parsing XML rectilinear grids 
	* Ascii
	* Binary
	* Base64

## Example

```rust
use ndarray::{Array1, Array2, Array3};
use vtk::{Spans2D, Mesh2D, Rectilinear2D};

let nx = 100;
let ny = 100;

// define all the points and their location in the grid
let x_locations : Vec<f64> = Array1::linspace(0., (nx * ny) as f64, nx*ny).to_vec();
let y_locations : Vec<f64> = x_locations.clone();

// create a mesh object from the point locations and specify that 
// you want mesh information to be encoded as binary (could also 
// use `vtk::Ascii` or `vtk::Base64`.
let mesh = Mesh2D::<vtk::Binary>::new(x_locations, y_locations);

// define the global location of this data in the domain
// most of the time, you want something like this:
let spans = Spans2D::new(nx, ny);

// create an object to describe the entire domain
let domain = Rectilinear2D::new(mesh, spans);

// now lets make a container that holds all the information 
// we want to store in the file
#[derive(vtk::DataArray)]
#[vtk_write(encoding="binary")] // could also be "ascii" or "base64"
pub struct OurData {
    pressure: vtk::Scalar2D,
    velocity: vtk::Field2D
}

let pressure =vtk::Scalar2D::new(Array2::ones((nx,ny)));
// Note here that the velocities u,v,w are switched between using the first 
// index. This is for memory performance when writing and reading arrays. 
// You must store your data like this if you expect it to be interpreted 
// by paraview correctly
let velocity =vtk::Field2D::new(Array3::ones((3, nx, ny)));

let data = OurData { pressure, velocity };

// in order to write to a file, the vtk object needs to have information on the 
// domain and the data as it exists in that domain.
let vtk_data = vtk::VtkData::new(domain, data);

let file = std::fs::File::create("./test_vtks/your_file.vtk").unwrap();
let writer = std::io::BufWriter::new(file);

// finally, write all the data to the file
vtk::write_vtk(writer, vtk_data);
```

## Deriving Traits
The implementation for [`DataArray`](crate::DataArray) and [`ParseArray`](crate::ParseArray) 
on all your types can be tedious. If you add a new member to your data struct,
you must also remember to add an additional call to `write_dataarray`.  Instead, you can derive 
the traits.

If you want to write data to a file derive the `DataArray` trait. If you are parsing
data from a file, derive the `ParseArray` trait. Both traits accept attributes for
code generation. All fields in the struct must implement the [FromBuffer](FromBuffer) trait.
If you stick to the types in [vtk::array](crate::array) such as [`Field3D`](Field3D) you wont have
much issue with this.

The `DataArray` derive accepts `vtk_write`. It is used to specify the encoding of
the data being written to the file. It defaults to binary encoding:

```rust
#[derive(vtk::DataArray)]
#[vtk_write(encoding="base64")] // could also be "binary" (default) and "ascii"
struct VelocityField {
    a: Vec<f64>,
    b: vtk::Field3D,
    c: vtk::Scalar3D
}
```

For deriving `ParseArray` you **must** specify what spans you are parsing:

```rust
#[derive(vtk::ParseArray)]
#[vtk_parse(spans="vtk::Spans3D")]
pub struct VelocityField {
    a: Vec<f64>,
    b: vtk::Field3D,
    c: vtk::Scalar3D
}
```

If you specify the wrong spans there will be a compiler error:

```rust,ignore
#[derive(vtk::ParseArray)]
// specified 2d geometry with 3d arrays in the struct
#[vtk_parse(spans="vtk::Spans2D")]
pub struct VelocityField {
    a: Vec<f64>,
    b: vtk::Field3D,
    c: vtk::Scalar3D
}
```

```bash
error[E0277]: the trait bound `Scalar3D: FromBuffer<Spans2D>` is not satisfied
 --> src/lib.rs:112:10
  |
4 | #[derive(vtk::ParseArray)]
  |          ^^^^^^^^^^^^^^^ the trait `FromBuffer<Spans2D>` is not implemented for `Scalar3D`
  |
  = help: the following implementations were found:
            <Scalar3D as FromBuffer<Spans3D>>
  = note: this error originates in the derive macro `vtk::ParseArray` (in Nightly builds, run with -Z macro-backtrace for more info)
```

If you forget to specify the spans with the attribute `vtk_parse` you will get a sligtly ambiguous compiler error:

```bash
error: proc-macro derive panicked
 --> src/lib.rs:114:10
  |
4 | #[derive(vtk::ParseArray)]
  |          ^^^^^^^^^^^^^^^
  |
  = help: message: called `Result::unwrap()` on an `Err` value: Error { kind: MissingField("spans"), locations: [], span: None }
```


If you are reading data from a file, then you 

## Encoding Sizes

The following describes the VTK file size for a base case of 6.0 megabytes of data. The binary
format has the lowest overhead while ascii has the highest. Conversely, the ascii files
are the most human readable while the base64 / binary files are not.

| Encoding Type        | Size (MB) | % increase |
|----------------------|-----------|------------|
| Raw Data (base case) | 6.0       | -          |
| Binary               | 6.1       | 1.67       |
| Base64               | 8.1       | 35.00      |
| Ascii                | 10.0      | 66.67      |

## Working with vectors

It is highly discouraged to work with raw vectors when reading and writing arrays of data. It is
usually preferable to use a wrapper around a `ndarray` array such as those in
[vtk::array](crate::array). If you insist on using vectors the following is important.

Since the data is usually ordered in a `Vec<_>` it is ambiguous how `vtk` expects the data to be inputted. This is 
best illustrated code:

```python,ignore
data_to_write_vtk = []

for k in range(0, NZ):
    for j in range(0, NY):
        for i in range(0, NX):
            data_to_write_vtk.append(u_velocity[i,j,k])
```

In other words, iterate though your data in this order: Z, Y, X. This is illustrated for rectilinear systems
in the [paraview VTK file format specification](https://kitware.github.io/vtk-examples/site/VTKFileFormats/):

![](./static/data_ordering.png)
