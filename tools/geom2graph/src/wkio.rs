use log::warn;
use std::io::{BufRead, BufReader, Read};
use std::marker::PhantomData;

/// Take WKT line-by-line and stream geos::Geometries
#[derive(Debug)]
pub struct WktDeserializer<'geom, R: Read> {
    reader: BufReader<R>,
    phantom: PhantomData<&'geom ()>,
}

impl<'geom, R: Read> WktDeserializer<'geom, R> {
    /// Create a new WktDeserializer from a BufReader
    pub fn from_reader(reader: BufReader<R>) -> WktDeserializer<'geom, R> {
        WktDeserializer {
            reader,
            phantom: PhantomData,
        }
    }
}

impl<'geom, R: Read> Iterator for WktDeserializer<'geom, R> {
    type Item = geos::Geometry<'geom>;

    fn next(&mut self) -> Option<Self::Item> {
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
    impl<'geom> WktDeserializer<'geom, &[u8]> {
        pub fn from_str(s: &str) -> WktDeserializer<&[u8]> {
            WktDeserializer {
                reader: BufReader::new(s.as_bytes()),
                phantom: PhantomData,
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
        while let Some(_) = geoms.next() {
            counter += 1;
        }
        assert_eq!(counter, 2);
    }

    #[test]
    fn test_skip_invalid() {
        // The first LINESTRING isn't on its own line, so it isn't parsed.
        // ditto for the second POINT.
        let wkt = "invalid wkt here\tLINESTRING(1 1, 0 0)\nPOINT(0 0)\nasdf POINT(1 1)  \t\n\t POINT(2 2)";
        let mut geoms = WktDeserializer::from_str(wkt);
        let mut counter = 0;
        while let Some(_) = geoms.next() {
            counter += 1;
        }
        assert_eq!(counter, 2);
    }

    #[test]
    fn test_no_infinite_recursion() {
        let wkt = "";
        let mut geoms = WktDeserializer::from_str(wkt);
        assert!(geoms.next() == None);
    }
}
