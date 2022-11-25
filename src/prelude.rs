//! Common traits and types that are useful for working with `vtk`
#![allow(unused_imports)]

pub use crate::data::VtkData;
pub use crate::traits::{
    Array, DataArray, Domain, Encode, FromBuffer, Numeric, ParseArray, ParseMesh, ParseSpan,
    Visitor, Span
};

pub(crate) use quick_xml::events::BytesEnd;
pub(crate) use quick_xml::events::BytesStart;
pub(crate) use quick_xml::events::Event;
pub(crate) use quick_xml::name::QName;
pub(crate) use quick_xml::reader::Reader;
pub(crate) use quick_xml::writer::Writer;

pub(crate) use crate::write_vtk::Precision;

pub(crate) use crate::{Ascii, Base64, Binary};
pub(crate) use crate::Error;
pub(crate) use std::cell::{RefCell, RefMut};
pub(crate) use std::io::Write;
pub(crate) use std::io::BufRead;

pub(crate) use crate::{parse, traits, write_vtk};

pub(crate) use derive_more::{Constructor, Deref, DerefMut, Display, From, Into};

pub(crate) use ndarray::{Array2, Array3, Array4};
