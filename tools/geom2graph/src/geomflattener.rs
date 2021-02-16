use geos::Geom;

// You have to wrap Geometry in order to implement a trait you don't own on a type you don't own.
// TODO: How can I hide the fact that this wrapper exists?
struct GeometryWrapper<'subgeom, 'geom>(pub &'geom geos::Geometry<'subgeom>);

pub struct GeometryIterator<'subgeom, 'geom> {
    geometry: GeometryWrapper<'subgeom, 'geom>,
    index: usize,
}

impl<'subgeom, 'geom> GeometryIterator<'subgeom, 'geom> {
    pub fn new(geometry: &'geom geos::Geometry<'subgeom>) -> GeometryIterator<'subgeom, 'geom> {
        GeometryIterator {
            index: 0,
            geometry: GeometryWrapper { 0: geometry },
        }
    }
}

// I would be remiss not to say the two lifetimes here were absolute ass to get right.
impl<'subgeom, 'geom> Iterator for GeometryIterator<'subgeom, 'geom> {
    type Item = geos::ConstGeometry<'subgeom, 'geom>;

    // TODO: What to do when the geom is a GeometryCollection containing another multi geometry?
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.geometry.0.get_num_geometries().unwrap_or(0) {
            return None;
        }
        let curr = self
            .geometry
            .0
            .get_geometry_n(self.index)
            .expect("Failed to get nth geometry");
        self.index += 1;
        return Some(curr);
    }
}

impl<'subgeom, 'geom> IntoIterator for GeometryWrapper<'subgeom, 'geom> {
    type Item = geos::ConstGeometry<'subgeom, 'geom>;
    type IntoIter = GeometryIterator<'subgeom, 'geom>;

    fn into_iter(self) -> Self::IntoIter {
        GeometryIterator::new(self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deepflatten::DeepFlattenExt;

    #[test]
    fn iter_point() {
        let geom = geos::Geometry::new_from_wkt("POINT(0 1)").expect("Failed to create POINT");
        let mut children = GeometryIterator::new(&geom);
        let child = children.next().unwrap();
        assert!(geom == child);

        let child = children.next();
        assert!(child == None);
    }

    #[test]
    fn iter_collection() {
        let geom = geos::Geometry::new_from_wkt(
            "GEOMETRYCOLLECTION (
                POINT(0 1),
                MULTIPOINT((2 3), (4 5))
                )",
        )
        .expect("Failed to create GEOMETRYCOLLECTION");
        let mut children = GeometryIterator::new(&geom);

        let child = children.next().unwrap();
        let expected = geos::Geometry::new_from_wkt("POINT(0 1)").unwrap();
        assert!(child == expected);

        // GeometryIterator isn't recursive.
        let child = children.next().unwrap();
        let expected = geos::Geometry::new_from_wkt("MULTIPOINT((2 3), (4 5))").unwrap();
        assert!(child == expected);

        let child = children.next();
        assert!(child == None);
    }

    #[test]
    fn into_iter_multipoint() {
        let geom = GeometryWrapper {
            0: &geos::Geometry::new_from_wkt("MULTIPOINT((0 1), (2 3))")
                .expect("Failed to create MULTIPOINT"),
        };
        let mut children = geom.into_iter();

        let child = children.next().unwrap();
        let expected = geos::Geometry::new_from_wkt("POINT(0 1)").unwrap();
        assert!(child == expected);

        let child = children.next().unwrap();
        let expected = geos::Geometry::new_from_wkt("POINT(2 3)").unwrap();
        assert!(child == expected);

        let child = children.next();
        assert!(child == None);
    }

    #[test]
    fn into_iter_collection() {
        let geom = geos::Geometry::new_from_wkt(
            "GEOMETRYCOLLECTION (
                POINT(0 1),
                MULTIPOINT((2 3), (4 5))
                )",
        )
        .expect("Failed to create GEOMETRYCOLLECTION");
        let wrapper = GeometryWrapper { 0: &geom };
        let mut children = wrapper.into_iter();

        let child = children.next().unwrap();
        let expected = geos::Geometry::new_from_wkt("POINT(0 1)").unwrap();
        assert!(child == expected);

        // GeometryIterator isn't recursive.
        let child = children.next().unwrap();
        let expected = geos::Geometry::new_from_wkt("MULTIPOINT((2 3), (4 5))").unwrap();
        assert!(child == expected);

        let child = children.next();
        assert!(child == None);
    }

    #[test]
    fn deep_point() {
        let geom = GeometryWrapper {
            0: &geos::Geometry::new_from_wkt("POINT(0 1)").expect("Failed to create POINT"),
        };
        let children = geom.into_iter();
        let flattened = children.deep_flatten();
        let collected: Vec<_> = flattened.collect();
        assert_eq!(collected.len(), 1);

        let expected = geos::Geometry::new_from_wkt("POINT(0 1)").expect("Failed to create POINT");
        assert!(collected[0] == expected);
    }

    #[test]
    fn deep_multipoint() {
        let geom = GeometryWrapper {
            0: &geos::Geometry::new_from_wkt("MULTIPOINT ((0 1), (2 3))")
                .expect("Failed to create MULTIPOINT"),
        };
        let children = geom.into_iter();
        let flattened = children.deep_flatten();
        let collected: Vec<_> = flattened.collect();
        assert_eq!(collected.len(), 2);

        let expected = geos::Geometry::new_from_wkt("POINT(0 1)").expect("Failed to create POINT");
        assert!(collected[0] == expected);

        let expected = geos::Geometry::new_from_wkt("POINT(2 3)").expect("Failed to create POINT");
        assert!(collected[1] == expected);
    }

    #[test]
    #[ignore] // This test fails because ConstGeometries are a bitch.
    fn deep_collection() {
        let geom = geos::Geometry::new_from_wkt(
            "GEOMETRYCOLLECTION (
                POINT(0 1),
                MULTIPOINT((2 3), (4 5))
                )",
        )
        .expect("Failed to create GEOMETRYCOLLECTION");
        let wrapper = GeometryWrapper { 0: &geom };
        let children = wrapper.into_iter();
        let flattened = children.deep_flatten();
        let collected: Vec<_> = flattened.collect();
        assert_eq!(collected.len(), 3); // TODO: This fails :(

        let expected = geos::Geometry::new_from_wkt("POINT(0 1)").unwrap();
        assert!(collected[0] == expected);

        let expected = geos::Geometry::new_from_wkt("POINT(2 3)").unwrap();
        assert!(collected[1] == expected);
        let expected = geos::Geometry::new_from_wkt("POINT(4 5)").unwrap();
        assert!(collected[2] == expected);
    }
}
