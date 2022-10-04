//! # Mesh Information
//!
//! A `Mesh` object contains information on the spacing of gridpoints in the domain of your system.
//! Conversely, a `Span` object specifies the number of grid points (and their offset in the
//! computational domain) is. The distinction is made in `vtk` because parsers _always_ have
//! access to full span information initially, while parsers must search through `.vtk` files to
//! find full information about the mesh.
//!
//! Putting a `Mesh` object together with a `Span` object gives you a full description of the
//! computational domain. These descriptions are most often mound in the `domain` field of
//! [VtkData](`crate::VtkData`).
//! Objects implementing the [Domain](`crate::Domain`) trait (such as [`Rectilinear3D`])
//!can be written to files with the [write_vtk](`crate::write_vtk()`) function.
//!
//!
//! ## Defining your own domain for writing files
//!
//!
//! If you need to specify  a computation domain that is not rectilinear, you can define
//! your own domains without issue. From the [Domain](`crate::Domain`) trait, you will
//! need to report to the library what the `Span` information is, as well as the bytes
//! in the desired encoding for the mesh coordinates. See the [parse](`crate::parse`)
//! documentation for information on built in routines to assist with this.
//!
//! ## Defining your own domain for reading files
//!
//! The domain trait is not important for writing vtk files. Instead, the
//! [ParseMesh](`crate::ParseMesh`) trait is used to provide type-level information
//! on what type your `Mesh` will use to parse information from coordinate arrays
//! or appended binary. The type is responsible for holding parts of the information
//! from the file is called a "`Visitor`". Incidentally, the data arrays you are reading
//! from the file automatically implement this `Visitor` type through the derive interface.
//!
//! The visitor has type-level information to what the spans of the file are. This
//! `Span` type must implement the [`ParseSpan`](`crate::ParseSpan`) trait in
//! instantiate itself from the file as it is being parsed.
//!
//! Lastly, your `Domain` type must implement `From` trait for a tuple of your
//! mesh type and your span type: `From<(Mesh, Span)>`.
//!
//!
//! ## Encoding types and their interactions with reading files
//!
//! `vtk` makes no assumptions about the layout of the data arrays (and the location of their
//! information), barring that they must appear in the same order in the file as they are in
//! the struct you are deriving on. Similarly, **the encoding type on a `Domain` type is not
//! used for parsing data**. If you have specified that your `Rectilinear3D` domain has a `Binary`
//! encoding, a file with `Ascii` coordinate arrays will be read without issue.

mod dim_2;
mod dim_3;

pub use dim_2::{Mesh2D, Rectilinear2D, Spans2D};
pub use dim_3::{Mesh3D, Rectilinear3D, Spans3D};

#[doc(hidden)]
pub use dim_3::Mesh3DVisitor;
