//! Common traits and types that are useful for working with `vtk`
#![allow(unused_imports)]

pub use crate::traits::{ParseArray, DataArray, Visitor, ParseSpan, ParseMesh, Domain, Array, Encode};
pub use crate::{EventWriter};
pub use crate::data::VtkData;

pub(crate) use crate::{Binary, Ascii, Base64};
pub(crate) use nom::IResult;
pub(crate) use crate::{Error, ParseError};
pub(crate) use std::io::Write;
pub(crate) use std::cell::{RefMut, RefCell};

pub(crate) use crate::{parse, traits, write_vtk};

