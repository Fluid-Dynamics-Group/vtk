//! container types for data to be read / written from files

mod scalar_2d;
mod scalar_3d;

mod field_2d;
mod field_3d;
mod vector;

use crate::traits::Array;
use std::io::Write;
use xml::writer::{EventWriter, XmlEvent};

pub use field_2d::Field2D;
