use log::warn;
use std::io::{BufRead, BufReader, Read};

/// Take WKT line-by-line and stream geos::Geometries
#[derive(Debug)]
pub struct WktDeserializer<R: Read> {
    reader: BufReader<R>,
}

impl<R: Read> WktDeserializer<R> {
    /// Create a new WktDeserializer from a BufReader
    pub fn from_reader(reader: BufReader<R>) -> WktDeserializer<R> {
        WktDeserializer { reader }
    }
}

// Need to use generic associated types (an unstable feature) to implement iterators where the
// items yielded have a lifetime apart from that of the iterator.
// See https://lukaskalbertodt.github.io/2018/08/03/solving-the-generalized-streaming-iterator-problem-without-gats.html
// TODO: This still doesn't play well with IntoIter.
pub trait GatIterator {
    type Item<'s>;
    fn next(&mut self) -> Option<Self::Item<'_>>;
}

impl<R: Read> GatIterator for WktDeserializer<R> {
    type Item<'g> = geos::Geometry<'g>;

    /// Get the next valid geos::Geometry from the input stream
    fn next(&mut self) -> Option<Self::Item<'_>> {
        let mut buffer = String::new();
        if let Ok(_) = self.reader.read_line(&mut buffer) {
            let buffer = buffer.trim();
            if buffer.len() > 0 {
                if let Ok(geometry) = geos::Geometry::new_from_wkt(buffer) {
                    return Some(geometry);
                } else {
                    warn!("Failed to deserialize geometry from '{}', skipping and trying to read next line.", buffer);
                    return self.next();
                }
            }
        }
        return None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geos::Geom;

    // Just for the tests.
    impl WktDeserializer<&[u8]> {
        pub fn from_str(s: &str) -> WktDeserializer<&[u8]> {
            WktDeserializer {
                reader: BufReader::new(s.as_bytes()),
            }
        }
    }

    #[test]
    fn test_geos_from_wkt() {
        let wkt = "POINT (1.0 2.5)";
        let geom = geos::Geometry::new_from_wkt(wkt).unwrap();
        assert_eq!(geom.geometry_type(), geos::GeometryTypes::Point);
    }

    #[test]
    fn test_from_str() {
        let wkt = "POINT (1.0 2.5)";
        let expected = geos::Geometry::new_from_wkt(wkt).unwrap();
        let mut geoms = WktDeserializer::from_str(wkt);

        let geom = geoms.next().unwrap();
        assert!(geom == expected); // Geometry doesn't derive Debug, so can't use assert_eq!
    }

    #[test]
    fn test_from_reader() {
        let wkt = "POINT (1.0 2.5)";
        let expected = geos::Geometry::new_from_wkt(wkt).unwrap();
        let str_reader = BufReader::new(wkt.as_bytes());
        let mut geoms = WktDeserializer::from_reader(str_reader);

        let geom = geoms.next().unwrap();
        assert!(geom == expected);
    }

    #[test]
    fn test_iterator_trait() {
        let wkt = "Point (0 0)\nPOINT (2 2)";
        let mut geoms = WktDeserializer::from_str(wkt);

        // TODO: It doesn't appear possible to implement std::iter::Iterator with Geometry<'g>s or
        // to add IntoIter for the GAT version that implements its own Iterator trait.
        // So for now we can't use for loops. Oh well.
        let mut counter = 0;
        while let Some(_geom) = geoms.next() {
            counter += 1;
        }
        assert_eq!(counter, 2);
    }

    struct CoordSeqIterator<'c> {
        index: usize,
        coords: geos::CoordSeq<'c>,
    }

    impl<'c> CoordSeqIterator<'c> {
        fn new(cs: geos::CoordSeq) -> CoordSeqIterator {
            CoordSeqIterator {
                index: 0,
                coords: cs,
            }
        }
    }

    /// Turn a CoordSeq into an iterator of geos::GeometryTypes::Points.
    impl<'c> Iterator for CoordSeqIterator<'c> {
        type Item = geos::Geometry<'c>;

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

    // TODO: Split this into different tests, and add a new one that tests whitespace and garbage
    // skipping.
    #[test]
    fn test_get_coords() {
        // NOTE: Polygon's MUST be closed. Geos won't implicitly close them for you.
        let wkt = "POINT (1 2)  \t \nasdf\n   LINESTRING(1 2, 3 4)    \n POLYGON((0 0, 1 1, 2 2, 3 3, 4 4, 0 0), (0 0, 1 1, 2 2, 0 0))";
        let mut geoms = WktDeserializer::from_str(wkt);

        {
            let point = geoms.next().expect("Failed to parse POINT");
            let coords = point.get_coord_seq().expect("Couldn't get POINT coord seq");
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
        {
            let line = geoms.next().expect("Failed to parse LINESTRING");
            let coords = line
                .get_coord_seq()
                .expect("Couldn't get LINESTRING coord seq");
            assert_eq!(coords.dimensions(), Ok(geos::CoordDimensions::TwoD));
            assert_eq!(coords.size(), Ok(2));

            // TODO: Why does _this_ have to be mutable when you can loop over the coords just
            // fine?
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
        {
            let poly = geoms.next().expect("Failed to parse POLYGON");
            let exterior = poly
                .get_exterior_ring()
                .expect("Couldn't get exterior ring");
            let coords = exterior
                .get_coord_seq()
                .expect("Couldn't get POLYGON coord seq");

            assert_eq!(coords.dimensions(), Ok(geos::CoordDimensions::TwoD));
            assert_eq!(coords.size(), Ok(6));

            let num_holes = poly
                .get_num_interior_rings()
                .expect("Couldn't get num holes");
            assert_eq!(num_holes, 1);

            let hole = poly
                .get_interior_ring_n(0)
                .expect("Couldn't get first interior hole");
            let coords = hole.get_coord_seq().expect("Couldn't get hole coord seq");
            assert_eq!(coords.dimensions(), Ok(geos::CoordDimensions::TwoD));
            assert_eq!(coords.size(), Ok(4));
        }
    }
}
