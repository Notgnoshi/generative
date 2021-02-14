use log::{debug, warn};
use std::convert::TryFrom;
use std::io::{BufRead, BufReader, Read};
use wkt::Wkt;

/// Deserializes WKT text into geometries.
#[derive(Debug)]
pub struct WktDeserializer<R: Read> {
    reader: BufReader<R>,
}

impl<R: Read> WktDeserializer<R> {
    /// Create a WktDeserializer from the given BufReader
    pub fn new(reader: BufReader<R>) -> WktDeserializer<R> {
        WktDeserializer { reader }
    }
}

// TODO: Figure out why the hell I have to specialize for &[u8]
impl WktDeserializer<&[u8]> {
    /// Create a WktDeserializer from a &[u8]...
    fn from_str<'a>(s: &'a str) -> WktDeserializer<&[u8]> {
        // What the actual fuck. This is the most obscene, obtuse, and stupid syntax.
        // My first impression of Rust was that it's death by syntax, and so far that's held up...
        let reader = BufReader::new(s.as_bytes());
        return WktDeserializer::new(reader);
    }
}

impl<R: Read> Iterator for WktDeserializer<R> {
    type Item = geo::Geometry<f64>;

    /// Consume as many lines from the internal BufReader as necessary to spit out a new geometry.
    fn next(&mut self) -> Option<Self::Item> {
        let mut buf = String::new();
        // We were able to read a line.
        if let Ok(bytes) = self.reader.read_line(&mut buf) {
            if bytes > 0 {
                let buf = buf.trim();
                if let Ok(geom) = Wkt::<f64>::from_str(&buf) {
                    debug!("deserialized: '{:?}' from '{}'", geom, buf);

                    if let Ok(geom) = geo::Geometry::try_from(geom) {
                        return Some(geom);
                    }
                } else {
                    warn!("Failed to deserialize '{}'", buf);
                    // Keep going until you succeed. This is necessary so that we keep going on bad
                    // input. (returning None signals the end)
                    // TODO: Maybe don't use recursion for this? Or perhaps at least use tail call
                    // recursion?
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
    use std::convert::TryInto;

    #[test]
    fn test_simple() {
        let input = "POINT( 0 0 )";
        let expected: geo::Point<f64> = (0.0, 0.0).into(); // The geo type

        let reader = BufReader::new(input.as_bytes());
        let mut deserializer = WktDeserializer::new(reader);

        let geom = deserializer.next().unwrap();

        assert_eq!(expected, geom.try_into().unwrap());

        let wkt_geom = deserializer.next();
        assert!(wkt_geom.is_none());
    }

    #[test]
    fn test_from_str() {
        let input = "POINT( 0 0 )";
        let expected: geo::Point<f64> = (0.0, 0.0).into(); // The geo type

        let mut deserializer = WktDeserializer::from_str(input);

        let geom = deserializer.next().unwrap();
        assert_eq!(expected, geom.try_into().unwrap());
    }

    #[test]
    fn test_no_infinite_recursion() {
        let input = "Nothing valid to see here.";
        let mut deserializer = WktDeserializer::from_str(input);

        let geom = deserializer.next();
        assert!(geom.is_none());
    }

    #[test]
    fn test_skip_invalid() {
        let input = "POINT( 0 0 )\nasdf\t\nPOINT( 1 1)";
        let expected1: geo::Point<f64> = (0.0, 0.0).into();
        let expected2: geo::Point<f64> = (1.0, 1.0).into();

        let mut deserializer = WktDeserializer::from_str(input);

        let geom = deserializer.next().unwrap();
        assert_eq!(expected1, geom.try_into().unwrap());

        let geom = deserializer.next().unwrap();
        assert_eq!(expected2, geom.try_into().unwrap());

        let geom = deserializer.next();
        assert!(geom.is_none());
    }

    #[test]
    fn test_match() {
        let input = "POINT( 0 0 )\nLINESTRING(0 0, 1 1)";
        let mut deserializer = WktDeserializer::from_str(input);

        // Really just testing that it compiles.
        let geom = deserializer.next().unwrap();
        match geom {
            geo::Geometry::Point(_) => {
                assert!(true);
            }
            geo::Geometry::LineString(_) => {
                assert!(false);
            }
            _ => {
                assert!(false);
            }
        }

        let geom = deserializer.next().unwrap();
        match geom {
            geo::Geometry::Point(_) => {
                assert!(false);
            }
            geo::Geometry::LineString(_) => {
                assert!(true);
            }
            _ => {
                assert!(false);
            }
        }
    }
}
