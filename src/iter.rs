use super::traits::PointData;

pub struct VtkIterator<D: PointData> {
    idx: usize,
    data: D,
}
impl<D> VtkIterator<D>
where
    D: PointData,
{
    pub(crate) fn new(data: D) -> Self {
        Self { idx: 0, data }
    }
}
impl<D> Iterator for VtkIterator<D>
where
    D: PointData,
{
    type Item = D::PointData;

    fn next(&mut self) -> Option<Self::Item> {
        let point_data = self.data.get_point_data(self.idx);

        self.idx += 1;
        point_data
    }
}
