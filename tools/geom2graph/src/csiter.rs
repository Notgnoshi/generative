use geos::Geom;

/// An iterator over the points of a geos::Geometry.
pub struct PointIterator<'c> {
    /// The current index of the wrapped coords.
    index: usize,

    /// The backing CoordSeq of the Geometry.
    /// TODO: Find a way to make this a reference to avoid unnecessarily copying potentially large
    /// geometries. I never thought I'd miss C++.
    coords: geos::CoordSeq<'c>,
}

impl<'c> PointIterator<'c> {
    /// Create a PointIterator from a geos::CoordSeq.
    /// You likely want to use PointIterator::new() instead.
    fn new_from_cs(cs: geos::CoordSeq) -> PointIterator {
        PointIterator {
            index: 0,
            coords: cs,
        }
    }

    /// Create a PointIterator from a geos::Geometry.
    /// TODO: Handle multi-geometries and geometry collections.
    /// NOTE: geos::Geometry::get_coord_seq() clones the underlying CoordSeq because of memory
    /// management :/
    pub fn new(geom: geos::Geometry) -> PointIterator {
        let coordinate_sequence = match geom.geometry_type() {
            geos::GeometryTypes::Polygon => {
                let exterior = geom
                    .get_exterior_ring()
                    .expect("Couldn't get POLYGON exterior ring");
                exterior.get_coord_seq()
            }
            _ => geom.get_coord_seq(),
        };

        // Instead of crashing on an unsupported geometry type (something that's not a POINT,
        // LINESTRING, or LINEARRING), create an empty PointIterator
        match coordinate_sequence {
            Ok(_) => PointIterator::new_from_cs(coordinate_sequence.unwrap()),
            _ => PointIterator::new_from_cs(
                geos::CoordSeq::new(0, geos::CoordDimensions::ThreeD).unwrap(),
            ),
        }
    }

    pub fn new_from_const_geom(geometry: geos::ConstGeometry<'c, '_>) -> PointIterator<'c> {
        let coordinate_sequence = match geometry.geometry_type() {
            geos::GeometryTypes::Polygon => {
                let exterior = geometry
                    .get_exterior_ring()
                    .expect("Couldn't get POLYGON exterior ring");
                exterior.get_coord_seq()
            }
            _ => geometry.get_coord_seq(),
        };

        // Instead of crashing on an unsupported geometry type (something that's not a POINT,
        // LINESTRING, or LINEARRING), create an empty PointIterator
        match coordinate_sequence {
            Ok(_) => PointIterator::new_from_cs(coordinate_sequence.unwrap()),
            _ => PointIterator::new_from_cs(
                geos::CoordSeq::new(0, geos::CoordDimensions::ThreeD).unwrap(),
            ),
        }
    }
}

impl<'c> Iterator for PointIterator<'c> {
    type Item = geos::Geometry<'c>; // TODO: Is there a way to specify that this is a Point?

    /// Get the next point from the Geometry.
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.coords.size().unwrap_or(0) {
            return None; // Indicate the end of the sequence.
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_cs() {
        let geom = geos::Geometry::new_from_wkt("POINT(1 2)").expect("Failed to create POINT");
        let points = PointIterator::new(geom);

        // Test the iterable interface.
        for point in points {
            assert_eq!(point.geometry_type(), geos::GeometryTypes::Point);

            let x = point.get_x().expect("Failed to get X");
            let y = point.get_y().expect("Failed to get Y");
            let z = point.get_z().expect("Failed to get Z");

            assert_eq!(x, 1.0);
            assert_eq!(y, 2.0);
            assert_eq!(z, 0.0);

            let c = point.get_num_coordinates();
            assert_eq!(c, Ok(1));

            let dim = point.get_coordinate_dimension();
            assert_eq!(dim, Ok(geos::Dimensions::ThreeD));

            let dim = point.get_num_dimensions();
            assert_eq!(dim, Ok(0)); // WTF?

            // BUG: This only checks the first two coordinates.
            assert!(point == geos::Geometry::new_from_wkt("POINT Z (1 2 999999999)").unwrap());
        }
    }

    #[test]
    fn test_pointz_cs() {
        let geom =
            geos::Geometry::new_from_wkt("Point Z (1 2 3)").expect("Failed to create POINT Z");
        let mut points = PointIterator::new(geom);
        let point = points.next().expect("Failed to get the first point");
        assert_eq!(point.geometry_type(), geos::GeometryTypes::Point);

        let x = point.get_x().expect("Failed to get X");
        let y = point.get_y().expect("Failed to get Y");
        let z = point.get_z().expect("Failed to get Z");

        assert_eq!(x, 1.0);
        assert_eq!(y, 2.0);
        assert_eq!(z, 3.0);
    }

    #[test]
    fn test_linestring_cs() {
        let geom = geos::Geometry::new_from_wkt("LINESTRING(1 2, 3 4)")
            .expect("Failed to create LINESTRING");
        let mut points = PointIterator::new(geom);

        let point = points.next().expect("Failed to get first point");
        assert_eq!(point.geometry_type(), geos::GeometryTypes::Point);
        let x = point.get_x().expect("Failed to get X");
        let y = point.get_y().expect("Failed to get Y");
        let z = point.get_z().expect("Failed to get Z");

        assert_eq!(x, 1.0);
        assert_eq!(y, 2.0);
        assert_eq!(z, 0.0);

        let point = points.next().expect("Failed to get first point");
        assert_eq!(point.geometry_type(), geos::GeometryTypes::Point);
        let x = point.get_x().expect("Failed to get X");
        let y = point.get_y().expect("Failed to get Y");
        let z = point.get_z().expect("Failed to get Z");

        assert_eq!(x, 3.0);
        assert_eq!(y, 4.0);
        assert_eq!(z, 0.0);
    }

    #[test]
    fn test_linestringz_cs() {
        let geom = geos::Geometry::new_from_wkt("LINESTRING Z(1 2 3, 4 5 6)")
            .expect("Failed to create LINESTRING Z");
        let mut points = PointIterator::new(geom);

        let point = points.next().expect("Failed to get first point");
        assert_eq!(point.geometry_type(), geos::GeometryTypes::Point);

        let x = point.get_x().expect("Failed to get X");
        let y = point.get_y().expect("Failed to get Y");
        let z = point.get_z().expect("Failed to get Z");

        assert_eq!(x, 1.0);
        assert_eq!(y, 2.0);
        assert_eq!(z, 3.0);

        let point = points.next().expect("Failed to get first point");
        assert_eq!(point.geometry_type(), geos::GeometryTypes::Point);

        let x = point.get_x().expect("Failed to get X");
        let y = point.get_y().expect("Failed to get Y");
        let z = point.get_z().expect("Failed to get Z");

        assert_eq!(x, 4.0);
        assert_eq!(y, 5.0);
        assert_eq!(z, 6.0);
    }

    #[test]
    fn test_polygon_cs() {
        let geom = geos::Geometry::new_from_wkt(
            "POLYGON((0 0, 1 1, 2 2, 3 3, 4 4, 0 0), (0 0, 1 1, 2 2, 0 0))",
        )
        .expect("Failed to create POLYGON");
        let mut points = PointIterator::new(geom);

        let point = points.next().expect("Failed to get first point");
        let x = point.get_x().expect("Failed to get X");
        let y = point.get_y().expect("Failed to get Y");
        let z = point.get_z().expect("Failed to get Z");
        assert_eq!(x, 0.0);
        assert_eq!(y, 0.0);
        assert_eq!(z, 0.0);

        let point = points.next().expect("Failed to get first point");
        let x = point.get_x().expect("Failed to get X");
        let y = point.get_y().expect("Failed to get Y");
        let z = point.get_z().expect("Failed to get Z");
        assert_eq!(x, 1.0);
        assert_eq!(y, 1.0);
        assert_eq!(z, 0.0);

        let point = points.next().expect("Failed to get first point");
        let x = point.get_x().expect("Failed to get X");
        let y = point.get_y().expect("Failed to get Y");
        let z = point.get_z().expect("Failed to get Z");
        assert_eq!(x, 2.0);
        assert_eq!(y, 2.0);
        assert_eq!(z, 0.0);

        let point = points.next().expect("Failed to get first point");
        let x = point.get_x().expect("Failed to get X");
        let y = point.get_y().expect("Failed to get Y");
        let z = point.get_z().expect("Failed to get Z");
        assert_eq!(x, 3.0);
        assert_eq!(y, 3.0);
        assert_eq!(z, 0.0);

        let point = points.next().expect("Failed to get first point");
        let x = point.get_x().expect("Failed to get X");
        let y = point.get_y().expect("Failed to get Y");
        let z = point.get_z().expect("Failed to get Z");
        assert_eq!(x, 4.0);
        assert_eq!(y, 4.0);
        assert_eq!(z, 0.0);

        let point = points.next().expect("Failed to get first point");
        let x = point.get_x().expect("Failed to get X");
        let y = point.get_y().expect("Failed to get Y");
        let z = point.get_z().expect("Failed to get Z");
        assert_eq!(x, 0.0);
        assert_eq!(y, 0.0);
        assert_eq!(z, 0.0);
    }

    #[test]
    fn test_linearring_cs() {
        let geom = geos::Geometry::new_from_wkt("LINEARRING (0 0, 1 1, 2 2, 0 0)")
            .expect("Failed to create LINEARRING");
        let mut points = PointIterator::new(geom);

        let point = points.next().expect("Failed to get first point");
        let x = point.get_x().expect("Failed to get X");
        let y = point.get_y().expect("Failed to get Y");
        let z = point.get_z().expect("Failed to get Z");
        assert_eq!(x, 0.0);
        assert_eq!(y, 0.0);
        assert_eq!(z, 0.0);

        let point = points.next().expect("Failed to get first point");
        let x = point.get_x().expect("Failed to get X");
        let y = point.get_y().expect("Failed to get Y");
        let z = point.get_z().expect("Failed to get Z");
        assert_eq!(x, 1.0);
        assert_eq!(y, 1.0);
        assert_eq!(z, 0.0);

        let point = points.next().expect("Failed to get first point");
        let x = point.get_x().expect("Failed to get X");
        let y = point.get_y().expect("Failed to get Y");
        let z = point.get_z().expect("Failed to get Z");
        assert_eq!(x, 2.0);
        assert_eq!(y, 2.0);
        assert_eq!(z, 0.0);

        let point = points.next().expect("Failed to get first point");
        let x = point.get_x().expect("Failed to get X");
        let y = point.get_y().expect("Failed to get Y");
        let z = point.get_z().expect("Failed to get Z");
        assert_eq!(x, 0.0);
        assert_eq!(y, 0.0);
        assert_eq!(z, 0.0);
    }

    #[test]
    #[ignore]
    fn test_multipoint_cs() {
        let geom = geos::Geometry::new_from_wkt("MULTIPOINT((0 0), (1 1))")
            .expect("Failed to create MULTIPOINT");
        let mut points = PointIterator::new(geom);

        let point = points.next().expect("Failed to get first point");
        let x = point.get_x().expect("Failed to get X");
        let y = point.get_y().expect("Failed to get Y");
        let z = point.get_z().expect("Failed to get Z");
        assert_eq!(x, 0.0);
        assert_eq!(y, 0.0);
        assert_eq!(z, 0.0);

        let point = points.next().expect("Failed to get first point");
        let x = point.get_x().expect("Failed to get X");
        let y = point.get_y().expect("Failed to get Y");
        let z = point.get_z().expect("Failed to get Z");
        assert_eq!(x, 1.0);
        assert_eq!(y, 1.0);
        assert_eq!(z, 0.0);
    }

    #[test]
    #[ignore]
    fn test_multilinestring_cs() {
        let geom =
            geos::Geometry::new_from_wkt("MULTILINESTRING((0 0, 1 1, 2 2), (3 3, 4 4, 5 5))")
                .expect("Failed to create MULTILINESTRING");
        let mut points = PointIterator::new(geom);
        let point = points.next().expect("Failed to get first point");
        let x = point.get_x().expect("Failed to get X");
        let y = point.get_y().expect("Failed to get Y");
        let z = point.get_z().expect("Failed to get Z");
        assert_eq!(x, 0.0);
        assert_eq!(y, 0.0);
        assert_eq!(z, 0.0);

        let point = points.next().expect("Failed to get first point");
        let x = point.get_x().expect("Failed to get X");
        let y = point.get_y().expect("Failed to get Y");
        let z = point.get_z().expect("Failed to get Z");
        assert_eq!(x, 1.0);
        assert_eq!(y, 1.0);
        assert_eq!(z, 0.0);

        let point = points.next().expect("Failed to get first point");
        let x = point.get_x().expect("Failed to get X");
        let y = point.get_y().expect("Failed to get Y");
        let z = point.get_z().expect("Failed to get Z");
        assert_eq!(x, 2.0);
        assert_eq!(y, 2.0);
        assert_eq!(z, 0.0);

        let point = points.next().expect("Failed to get first point");
        let x = point.get_x().expect("Failed to get X");
        let y = point.get_y().expect("Failed to get Y");
        let z = point.get_z().expect("Failed to get Z");
        assert_eq!(x, 3.0);
        assert_eq!(y, 3.0);
        assert_eq!(z, 0.0);

        let point = points.next().expect("Failed to get first point");
        let x = point.get_x().expect("Failed to get X");
        let y = point.get_y().expect("Failed to get Y");
        let z = point.get_z().expect("Failed to get Z");
        assert_eq!(x, 4.0);
        assert_eq!(y, 4.0);
        assert_eq!(z, 0.0);

        let point = points.next().expect("Failed to get first point");
        let x = point.get_x().expect("Failed to get X");
        let y = point.get_y().expect("Failed to get Y");
        let z = point.get_z().expect("Failed to get Z");
        assert_eq!(x, 5.0);
        assert_eq!(y, 5.0);
        assert_eq!(z, 0.0);
    }

    #[test]
    #[ignore]
    fn test_multipolygon_cs() {
        let geom = geos::Geometry::new_from_wkt(
            "MULTIPOLYGON(((0 0, 1 1, 2 2, 0 0)), ((2 2, 3 3, 4 4, 2 2), (0 0, 1 1, 2 2, 0 0)))",
        )
        .expect("Failed to create MULTIPOLYGON");
        let mut points = PointIterator::new(geom);
        let point = points.next().expect("Failed to get first point");
        let x = point.get_x().expect("Failed to get X");
        let y = point.get_y().expect("Failed to get Y");
        let z = point.get_z().expect("Failed to get Z");
        assert_eq!(x, 0.0);
        assert_eq!(y, 0.0);
        assert_eq!(z, 0.0);

        let point = points.next().expect("Failed to get first point");
        let x = point.get_x().expect("Failed to get X");
        let y = point.get_y().expect("Failed to get Y");
        let z = point.get_z().expect("Failed to get Z");
        assert_eq!(x, 1.0);
        assert_eq!(y, 1.0);
        assert_eq!(z, 0.0);

        let point = points.next().expect("Failed to get first point");
        let x = point.get_x().expect("Failed to get X");
        let y = point.get_y().expect("Failed to get Y");
        let z = point.get_z().expect("Failed to get Z");
        assert_eq!(x, 2.0);
        assert_eq!(y, 2.0);
        assert_eq!(z, 0.0);

        let point = points.next().expect("Failed to get first point");
        let x = point.get_x().expect("Failed to get X");
        let y = point.get_y().expect("Failed to get Y");
        let z = point.get_z().expect("Failed to get Z");
        assert_eq!(x, 0.0);
        assert_eq!(y, 0.0);
        assert_eq!(z, 0.0);

        let point = points.next().expect("Failed to get first point");
        let x = point.get_x().expect("Failed to get X");
        let y = point.get_y().expect("Failed to get Y");
        let z = point.get_z().expect("Failed to get Z");
        assert_eq!(x, 2.0);
        assert_eq!(y, 2.0);
        assert_eq!(z, 0.0);

        let point = points.next().expect("Failed to get first point");
        let x = point.get_x().expect("Failed to get X");
        let y = point.get_y().expect("Failed to get Y");
        let z = point.get_z().expect("Failed to get Z");
        assert_eq!(x, 3.0);
        assert_eq!(y, 3.0);
        assert_eq!(z, 0.0);

        let point = points.next().expect("Failed to get first point");
        let x = point.get_x().expect("Failed to get X");
        let y = point.get_y().expect("Failed to get Y");
        let z = point.get_z().expect("Failed to get Z");
        assert_eq!(x, 4.0);
        assert_eq!(y, 4.0);
        assert_eq!(z, 0.0);

        let point = points.next().expect("Failed to get first point");
        let x = point.get_x().expect("Failed to get X");
        let y = point.get_y().expect("Failed to get Y");
        let z = point.get_z().expect("Failed to get Z");
        assert_eq!(x, 2.0);
        assert_eq!(y, 2.0);
        assert_eq!(z, 0.0);

        let point = points.next();
        assert!(
            point == None,
            "Expected that we don't iterate over the second polygon's hole points"
        );
    }

    #[test]
    #[ignore]
    fn test_geometrycollection_cs() {
        let geom = geos::Geometry::new_from_wkt(
            "GEOMETRYCOLLECTION(POINT(0 0), LINESTRING(0 0, 1 1, 2 2), MULTIPOINT((0 0), (1 1)))",
        )
        .expect("Failed to create GEOMETRYCOLLECTION");
        let mut points = PointIterator::new(geom);

        let point = points.next().expect("Failed to get first point");
        let x = point.get_x().expect("Failed to get X");
        let y = point.get_y().expect("Failed to get Y");
        let z = point.get_z().expect("Failed to get Z");
        assert_eq!(x, 0.0);
        assert_eq!(y, 0.0);
        assert_eq!(z, 0.0);

        let point = points.next().expect("Failed to get first point");
        let x = point.get_x().expect("Failed to get X");
        let y = point.get_y().expect("Failed to get Y");
        let z = point.get_z().expect("Failed to get Z");
        assert_eq!(x, 0.0);
        assert_eq!(y, 0.0);
        assert_eq!(z, 0.0);

        let point = points.next().expect("Failed to get first point");
        let x = point.get_x().expect("Failed to get X");
        let y = point.get_y().expect("Failed to get Y");
        let z = point.get_z().expect("Failed to get Z");
        assert_eq!(x, 1.0);
        assert_eq!(y, 1.0);
        assert_eq!(z, 0.0);

        let point = points.next().expect("Failed to get first point");
        let x = point.get_x().expect("Failed to get X");
        let y = point.get_y().expect("Failed to get Y");
        let z = point.get_z().expect("Failed to get Z");
        assert_eq!(x, 2.0);
        assert_eq!(y, 2.0);
        assert_eq!(z, 0.0);

        let point = points.next().expect("Failed to get first point");
        let x = point.get_x().expect("Failed to get X");
        let y = point.get_y().expect("Failed to get Y");
        let z = point.get_z().expect("Failed to get Z");
        assert_eq!(x, 0.0);
        assert_eq!(y, 0.0);
        assert_eq!(z, 0.0);

        let point = points.next().expect("Failed to get first point");
        let x = point.get_x().expect("Failed to get X");
        let y = point.get_y().expect("Failed to get Y");
        let z = point.get_z().expect("Failed to get Z");
        assert_eq!(x, 1.0);
        assert_eq!(y, 1.0);
        assert_eq!(z, 0.0);
    }
}
