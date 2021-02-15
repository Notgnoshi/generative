use geos::Geom;

pub struct GeometryFlattener<'subgeom, 'geom> {
    index: usize,
    geometry: &'geom geos::Geometry<'subgeom>,
}

impl<'subgeom, 'geom> GeometryFlattener<'subgeom, 'geom> {
    // TODO: Perhaps this is where we recursively flatten, and then the GeometryFlattener holds a
    // Vec of single geometries?
    fn new(geometry: &'geom geos::Geometry<'subgeom>) -> GeometryFlattener<'subgeom, 'geom> {
        GeometryFlattener { index: 0, geometry }
    }
}

// I would be remiss not to say the two lifetimes here were absolute ass to get right.
impl<'subgeom, 'geom> Iterator for GeometryFlattener<'subgeom, 'geom> {
    type Item = geos::ConstGeometry<'subgeom, 'geom>;

    // TODO: What to do when the geom is a GeometryCollection containing another multi geometry?
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.geometry.get_num_geometries().unwrap_or(0) {
            return None;
        }
        let curr = self
            .geometry
            .get_geometry_n(self.index)
            .expect("Failed to get nth geometry");
        self.index += 1;
        return Some(curr);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Just a test bed while I work on the Iterator trait :/
    fn get_first_child<'subgeom, 'geom>(
        geom: &'geom geos::Geometry<'subgeom>,
    ) -> geos::ConstGeometry<'subgeom, 'geom> {
        let index = 0;
        let child = geom.get_geometry_n(index).expect("");

        return child;
    }

    #[test]
    fn test_one() {
        let geom = geos::Geometry::new_from_wkt("POINT(0 1)").expect("");
        let child = get_first_child(&geom);
        assert!(geom == child);
    }

    #[test]
    fn test_geom_flattener_point() {
        let geom = geos::Geometry::new_from_wkt("POINT(0 1)").expect("");
        let mut children = GeometryFlattener::new(&geom);
        let child = children.next().unwrap();
        assert!(geom == child);
    }
}
