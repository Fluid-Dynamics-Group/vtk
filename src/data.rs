#[derive(Debug, Default, Clone, PartialEq)]
/// Container type to read and write vtk files from.
///
/// `VtkData` contains two objects: a `D` in data and a `DOMAIN`. For reading files,
/// `D` must implement `ParseArray` which is most easily accomplished with the derive
/// proc macro.
///
/// For writing files, `domain` must implement the [`Domain`](`crate::Domain`) trait and `data` must implement
/// the `DataArray` trait  which can be derived for your type. The `Domain` trait is already
/// implemented for two container types for rectilinear data:
/// [Rectilinear3D](`crate::Rectilinear3D`) and [Rectilinear2D](`crate::Rectilinear2D`) (depending
/// on the dimensionality of your data).
pub struct VtkData<DOMAIN, D> {
    pub domain: DOMAIN,
    pub data: D,
}

impl<DOMAIN, D> VtkData<DOMAIN, D> {
    /// Construct a `vtk` container for writing to a file
    pub fn new(domain: DOMAIN, data: D) -> VtkData<DOMAIN, D> {
        VtkData { domain, data }
    }

    /// change the datatype of the data stored in this container while leaving the
    /// domain information constant
    pub fn new_data<T>(self, new_data: T) -> VtkData<DOMAIN, T> {
        VtkData {
            domain: self.domain,
            data: new_data,
        }
    }
}
