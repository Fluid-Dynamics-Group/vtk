use std::io::Write;
use xml::EventWriter;

pub trait DataArray {
    fn write_dataarray<W: Write>(self, writer: &mut EventWriter<W>) -> Result<(), crate::Error>;
}

pub trait Extender {
    type Extender;
    fn extend_all(self, extender: &mut Self::Extender);
}

pub trait PointData {
    /// if Data contains a field of Vec<T>, this is just the T
    type PointData;
    fn get_point_data(&self, idx: usize) -> Option<Self::PointData>;
}

pub trait Combine {
    fn total_procs(&self) -> usize;
    //
    fn x_dims(&self) -> (usize, usize);
    fn y_dims(&self) -> (usize, usize);
    fn z_dims(&self) -> (usize, usize);
    //
    fn x_locations(&self) -> Vec<f64>;
    fn y_locations(&self) -> Vec<f64>;
    fn z_locations(&self) -> Vec<f64>;
}

pub trait ParseDataArray {
    fn parse_dataarrays(
        data: &str,
        span_info: &super::LocationSpans,
    ) -> Result<Self, super::xml_parse::NomErrorOwned>
    where
        Self: Sized;
}

//x_start: data[0].data.spans.x_start,
//x_end: data[total_procs - 1].data.spans.x_end,
//y_start: span_info.y_start,
//y_end: span_info.y_end,
//z_start: span_info.z_start,
//z_end: span_info.z_end,

pub trait Data: std::fmt::Debug + Default + Clone + PartialEq {}
