use geos::Geom;

// TODO: Rename to PointIterator
pub struct CoordSeqIterator<'c> {
    index: usize,
    coords: geos::CoordSeq<'c>,
}

impl<'c> CoordSeqIterator<'c> {
    // TODO: Is there a way to say this one is private?
    fn new(cs: geos::CoordSeq) -> CoordSeqIterator {
        CoordSeqIterator {
            index: 0,
            coords: cs,
        }
    }

    fn new_from_geometry(geom: geos::Geometry) -> CoordSeqIterator {
        // TODO: This only works for select geometry types. Need to match on the GeometryType
        // Will require recursion to create an iterator for GeometryCollections
        CoordSeqIterator {
            index: 0,
            coords: geom.get_coord_seq().expect("Failed to create CoordSeq from Geometry"),
        }
    }
}

/// Turn a CoordSeq into an iterator of geos::GeometryTypes::Points.
impl<'c> Iterator for CoordSeqIterator<'c> {
    type Item = geos::Geometry<'c>; // TODO: Is there a way to specify that this is a Point?

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.coords.size().unwrap() {
            return None;
        }

        // Implicitly convert all geometries to 3D.
        let x = self.coords.get_x(self.index).unwrap_or(0.0);
        let y = self.coords.get_y(self.index).unwrap_or(0.0);
        // get_z() panics if the geometry isn't 3D, even with .unwrap_or()...
        let z = if self.coords.dimensions().unwrap() == geos::CoordDimensions::ThreeD {
            self.coords.get_z(self.index).unwrap_or(0.0)
        } else {
            0.0
        };
        self.index += 1;

        let mut cs = geos::CoordSeq::new(1, geos::CoordDimensions::ThreeD)
            .expect("Failed to create new CoordSeq");

        cs.set_x(0, x).expect("Failed to set X");
        cs.set_y(0, y).expect("Failed to set Y");
        cs.set_z(0, z).expect("Failed to set Z");

        // Finally, we can create a new Point from the CoordSequence...
        return Some(geos::Geometry::create_point(cs).expect("Failed to create point"));

        // return None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_cs() {
        let geom = geos::Geometry::new_from_wkt("POINT(1 2)").expect("Failed to create POINT");
        // TODO: This should be handled by CoordSeqIterator
        let coords = geom.get_coord_seq().expect("Failed to get POINT coord seq");
        assert_eq!(coords.dimensions(), Ok(geos::CoordDimensions::TwoD));
        assert_eq!(coords.size(), Ok(1));

        let coords = CoordSeqIterator::new(coords);
        // Test the iterable interface.
        for coord in coords {
            assert_eq!(coord.geometry_type(), geos::GeometryTypes::Point);

            let x = coord.get_x().expect("Failed to get X");
            let y = coord.get_y().expect("Failed to get Y");
            let z = coord.get_z().expect("Failed to get Z");

            assert_eq!(x, 1.0);
            assert_eq!(y, 2.0);
            assert_eq!(z, 0.0);

            let c = coord.get_num_coordinates();
            assert_eq!(c, Ok(1));

            let dim = coord.get_coordinate_dimension();
            assert_eq!(dim, Ok(geos::Dimensions::ThreeD));

            let dim = coord.get_num_dimensions();
            assert_eq!(dim, Ok(0)); // WTF?

            // BUG: This only checks the first two coordinates.
            assert!(coord == geos::Geometry::new_from_wkt("POINT Z (1 2 999999999)").unwrap());
        }
    }

    #[test]
    fn test_linestring_cs() {
        let geom = geos::Geometry::new_from_wkt("LINESTRING(1 2, 3 4)")
            .expect("Failed to create LINESTRING");
        let coords = geom
            .get_coord_seq()
            .expect("Couldn't get LINESTRING coord seq");
        assert_eq!(coords.dimensions(), Ok(geos::CoordDimensions::TwoD));
        assert_eq!(coords.size(), Ok(2));

        // TODO: Why does _this_ have to be mutable when you can loop over the coords just fine?
        let mut coords = CoordSeqIterator::new(coords);

        let coord = coords.next().expect("Failed to get first point");
        assert_eq!(coord.geometry_type(), geos::GeometryTypes::Point);
        let x = coord.get_x().expect("Failed to get X");
        let y = coord.get_y().expect("Failed to get Y");
        let z = coord.get_z().expect("Failed to get Z");

        assert_eq!(x, 1.0);
        assert_eq!(y, 2.0);
        assert_eq!(z, 0.0);

        let coord = coords.next().expect("Failed to get first point");
        assert_eq!(coord.geometry_type(), geos::GeometryTypes::Point);
        let x = coord.get_x().expect("Failed to get X");
        let y = coord.get_y().expect("Failed to get Y");
        let z = coord.get_z().expect("Failed to get Z");

        assert_eq!(x, 3.0);
        assert_eq!(y, 4.0);
        assert_eq!(z, 0.0);
    }

    #[test]
    fn test_polygon_cs() {
        let geom = geos::Geometry::new_from_wkt(
            "POLYGON((0 0, 1 1, 2 2, 3 3, 4 4, 0 0), (0 0, 1 1, 2 2, 0 0))",
        )
        .expect("Failed to create POLYGON");
        let exterior = geom
            .get_exterior_ring()
            .expect("Couldn't get exterior ring");
        let coords = exterior
            .get_coord_seq()
            .expect("Couldn't get POLYGON coord seq");

        assert_eq!(coords.dimensions(), Ok(geos::CoordDimensions::TwoD));
        assert_eq!(coords.size(), Ok(6));

        let num_holes = geom
            .get_num_interior_rings()
            .expect("Couldn't get num holes");
        assert_eq!(num_holes, 1);

        let hole = geom
            .get_interior_ring_n(0)
            .expect("Couldn't get first interior hole");
        let coords = hole.get_coord_seq().expect("Couldn't get hole coord seq");
        assert_eq!(coords.dimensions(), Ok(geos::CoordDimensions::TwoD));
        assert_eq!(coords.size(), Ok(4));
    }

    #[test]
    fn test_linearring_cs() {}

    #[test]
    fn test_multipoint_cs() {}

    #[test]
    fn test_multilinestring_cs() {}

    #[test]
    fn test_multipolygon_cs() {}

    #[test]
    fn test_geometrycollection_cs() {}
}
