use super::data;

pub struct VtkIterator {
    idx: usize,
    data: data::SpanData,
}
impl VtkIterator {
    pub(crate) fn new(data: data::SpanData) -> Self {
        Self { idx: 0, data }
    }
}
impl Iterator for VtkIterator {
    type Item = PointData;

    fn next(&mut self) -> Option<Self::Item> {
        let rho = self.data.rho.get(self.idx);
        let u = self.data.u.get(self.idx);
        let v = self.data.v.get(self.idx);
        let w = self.data.w.get(self.idx);
        let energy = self.data.energy.get(self.idx);

        let out = rho
            .zip(u)
            .zip(v)
            .zip(w)
            .zip(energy)
            .and_then(|((((rho, u), v), w), energy)| {
                Some(PointData {
                    rho: *rho,
                    u: *u,
                    v: *v,
                    w: *w,
                    energy: *energy,
                })
            });
        self.idx += 1;
        out
    }
}

pub struct PointData {
    #[allow(dead_code)]
    pub(crate) rho: f64,
    #[allow(dead_code)]
    pub(crate) u: f64,
    #[allow(dead_code)]
    pub(crate) v: f64,
    #[allow(dead_code)]
    pub(crate) w: f64,
    #[allow(dead_code)]
    pub(crate) energy: f64,
}
