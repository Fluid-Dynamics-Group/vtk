#[derive(Debug, Default, Clone, PartialEq)]
pub struct VtkData<DOMAIN, D> {
    pub domain: DOMAIN,
    pub data: D,
}

impl<DOMAIN, D> VtkData<DOMAIN, D> {
    /// Construct a `vtk` container for writing to a file
    pub fn new(domain: DOMAIN, data: D) -> VtkData<DOMAIN, D> {
        VtkData { domain, data }
    }

    /// change the datatype of the data stored in this container
    pub fn new_data<T>(self, new_data: T) -> VtkData<DOMAIN, T> {
        VtkData {
            domain: self.domain,
            data: new_data,
        }
    }
}
