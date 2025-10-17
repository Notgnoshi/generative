use std::io::{BufRead, BufReader, Lines, Read, Write};
use std::str::FromStr;

use geo::{
    CoordNum, Geometry, GeometryCollection, Line, LineString, MultiLineString, MultiPoint,
    MultiPolygon, Point, Polygon, Rect, Triangle,
};
use wkt::{ToWkt, Wkt};

#[derive(PartialEq, Clone, Debug)]
pub enum SvgStyle {
    PointRadius(f64),
    Stroke(String),
    StrokeWidth(f64),
    StrokeDasharray(String),
    Fill(String),
}

fn wkt_inner<'a>(prefix: &'a str, s: &'a str) -> &'a str {
    if let Some(s) = s.strip_prefix(prefix) {
        let s = s.trim();
        let s = s.trim_start_matches('(');
        s.trim_end_matches(')')
    } else {
        ""
    }
}

impl TryFrom<&str> for SvgStyle {
    type Error = String;

    // This isn't a very good parser (there's lots of edge cases it doesn't handle) but for now it
    // doesn't have to be!
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let s = s.to_uppercase();

        if s.starts_with("POINTRADIUS") {
            let inner = wkt_inner("POINTRADIUS", &s);
            let radius = match f64::from_str(inner) {
                Ok(radius) => radius,
                Err(_) => return Err(format!("Failed to parse point radius from '{inner}'")),
            };
            return Ok(SvgStyle::PointRadius(radius));
        } else if s.starts_with("STROKEWIDTH") {
            let inner = wkt_inner("STROKEWIDTH", &s);
            let width = match f64::from_str(inner) {
                Ok(width) => width,
                Err(_) => return Err(format!("Failed to parse stroke width from '{inner}'")),
            };
            return Ok(SvgStyle::StrokeWidth(width));
        } else if s.starts_with("STROKEDASHARRAY") {
            let inner = wkt_inner("STROKEDASHARRAY", &s);
            return Ok(SvgStyle::StrokeDasharray(inner.into()));
        } else if s.starts_with("STROKE") {
            let inner = wkt_inner("STROKE", &s);
            return Ok(SvgStyle::Stroke(inner.to_lowercase()));
        } else if s.starts_with("FILL") {
            let inner = wkt_inner("FILL", &s);
            return Ok(SvgStyle::Fill(inner.to_lowercase()));
        }

        Err(format!("Failed to parse SVG style from '{s}'"))
    }
}

#[derive(PartialEq, Clone, Debug)]
pub enum GeometryAndStyle<T: CoordNum = f64> {
    Point(Point<T>),
    Line(Line<T>),
    LineString(LineString<T>),
    Polygon(Polygon<T>),
    MultiPoint(MultiPoint<T>),
    MultiLineString(MultiLineString<T>),
    MultiPolygon(MultiPolygon<T>),
    GeometryCollection(GeometryCollection<T>),
    Rect(Rect<T>),
    Triangle(Triangle<T>),

    Style(SvgStyle),
}

impl<T: CoordNum> From<Geometry<T>> for GeometryAndStyle<T> {
    fn from(x: Geometry<T>) -> Self {
        match x {
            Geometry::Point(p) => GeometryAndStyle::Point(p),
            Geometry::Line(l) => GeometryAndStyle::Line(l),
            Geometry::LineString(l) => GeometryAndStyle::LineString(l),
            Geometry::Polygon(p) => GeometryAndStyle::Polygon(p),
            Geometry::MultiPoint(m) => GeometryAndStyle::MultiPoint(m),
            Geometry::MultiLineString(m) => GeometryAndStyle::MultiLineString(m),
            Geometry::MultiPolygon(m) => GeometryAndStyle::MultiPolygon(m),
            Geometry::GeometryCollection(g) => GeometryAndStyle::GeometryCollection(g),
            Geometry::Rect(r) => GeometryAndStyle::Rect(r),
            Geometry::Triangle(t) => GeometryAndStyle::Triangle(t),
        }
    }
}

impl<T: CoordNum> From<GeometryAndStyle<T>> for Geometry<T> {
    fn from(x: GeometryAndStyle<T>) -> Self {
        match x {
            GeometryAndStyle::Point(p) => Geometry::Point(p),
            GeometryAndStyle::Line(l) => Geometry::Line(l),
            GeometryAndStyle::LineString(l) => Geometry::LineString(l),
            GeometryAndStyle::Polygon(p) => Geometry::Polygon(p),
            GeometryAndStyle::MultiPoint(m) => Geometry::MultiPoint(m),
            GeometryAndStyle::MultiLineString(m) => Geometry::MultiLineString(m),
            GeometryAndStyle::MultiPolygon(m) => Geometry::MultiPolygon(m),
            GeometryAndStyle::GeometryCollection(g) => Geometry::GeometryCollection(g),
            GeometryAndStyle::Rect(r) => Geometry::Rect(r),
            GeometryAndStyle::Triangle(t) => Geometry::Triangle(t),
            GeometryAndStyle::Style(_) => panic!("Cannot convert a STYLE into a geometry"),
        }
    }
}

impl<T: CoordNum> TryFrom<Wkt<T>> for GeometryAndStyle<T>
where
    GeometryAndStyle<T>: From<Geometry>,
{
    type Error = wkt::conversion::Error;

    fn try_from(wkt: Wkt<T>) -> Result<Self, Self::Error> {
        let geometry = Geometry::try_from(wkt)?;
        Ok(geometry.into())
    }
}

pub fn read_geometries<R>(reader: R) -> Box<dyn Iterator<Item = Geometry<f64>>>
where
    R: Read + 'static,
{
    Box::new(read_wkt_geometries(reader))
}

pub fn write_geometries<W, G>(writer: W, geometries: G) -> eyre::Result<()>
where
    W: Write,
    G: IntoIterator<Item = Geometry<f64>>,
{
    write_wkt_geometries(writer, geometries)
}

pub struct WktGeometries<R>
where
    R: Read,
{
    lines: Lines<BufReader<R>>,
}

pub struct WktGeometriesAndStyles<R>
where
    R: Read,
{
    lines: Lines<BufReader<R>>,
}

impl<R> Iterator for WktGeometries<R>
where
    R: Read,
{
    type Item = Geometry<f64>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.lines.next() {
            Some(Ok(line)) => match Wkt::<f64>::from_str(line.as_str()) {
                Ok(geometry) => match geometry.try_into() {
                    Ok(geometry) => Some(geometry),
                    Err(e) => {
                        tracing::warn!("Failed to convert '{line}' to geo geometry: {e:?}");
                        None
                    }
                },
                Err(e) => {
                    tracing::warn!("Failed to parse '{line}' as WKT: {e:?}");
                    None
                }
            },
            Some(Err(e)) => {
                tracing::warn!("Failed to read line: {e:?}");
                None
            }
            None => None,
        }
    }
}

impl<R> Iterator for WktGeometriesAndStyles<R>
where
    R: Read,
{
    type Item = GeometryAndStyle<f64>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.lines.next() {
            Some(Ok(line)) => match Wkt::<f64>::from_str(line.as_str()) {
                Ok(geometry) => match geometry.try_into() {
                    Ok(geometry) => Some(geometry),
                    Err(e) => {
                        tracing::warn!("Failed to convert '{line}' to geo geometry: {e:?}");
                        None
                    }
                },
                Err(e) => match SvgStyle::try_from(line.as_str()) {
                    Ok(style) => Some(GeometryAndStyle::Style(style)),
                    Err(ee) => {
                        tracing::warn!(
                            "Failed to parse '{line}' as WKT: {e:?} and as SVG STYLE: {ee:?}"
                        );
                        None
                    }
                },
            },
            Some(Err(e)) => {
                tracing::warn!("Failed to read line: {e:?}");
                None
            }
            None => None,
        }
    }
}

/// Return an iterator to the WKT geometries passed in through the given BufReader
///
/// Expects one geometry per line (LF or CRLF). Parsing any given line ends after either the first
/// failure or the first geometry yielded, whichever comes first. That is, a line can have trailing
/// garbage, but not leading garbage.
pub fn read_wkt_geometries<R>(reader: R) -> WktGeometries<R>
where
    R: Read,
{
    WktGeometries {
        // TODO: Is there a nice way to implement whitespace-separated geometries?
        lines: BufReader::new(reader).lines(),
    }
}

/// Write the given geometries with the given Writer in WKT format
///
/// Each geometry will be written on its own line.
pub fn write_wkt_geometries<W, G>(mut writer: W, geometries: G) -> eyre::Result<()>
where
    W: Write,
    G: IntoIterator<Item = Geometry<f64>>,
{
    for geometry in geometries {
        let wkt_geom = geometry.to_wkt();
        writeln!(writer, "{wkt_geom}")?;
    }
    Ok(())
}

pub fn read_wkt_geometries_and_styles<R>(reader: R) -> WktGeometriesAndStyles<R>
where
    R: Read,
{
    WktGeometriesAndStyles {
        lines: BufReader::new(reader).lines(),
    }
}

#[cfg(test)]
mod tests {
    use geo::{Geometry, Point};

    use super::*;

    #[test]
    fn test_read_simple_point() {
        let input = b"POINT(1 1)";
        let mut geometries = read_wkt_geometries(&input[..]);
        let geometry = geometries.next();
        assert_ne!(geometry, None);

        let geometry = geometry.unwrap();
        let point: Result<Point<f64>, _> = geometry.try_into();
        assert!(point.is_ok());
        let point = point.unwrap();

        let expected = Point::new(1.0, 1.0);
        assert_eq!(point, expected);
    }

    #[test]
    fn test_empty() {
        let input = b"";
        let mut geometries = read_wkt_geometries(&input[..]);
        assert_eq!(geometries.next(), None);
    }

    #[test]
    fn test_nothing_but_garbage() {
        let input = b"garbage";
        let mut geometries = read_wkt_geometries(&input[..]);
        assert_eq!(geometries.next(), None);
    }

    #[test]
    fn test_each_geometry_must_be_on_its_own_line() {
        let input = b"POINT(1 1)\nPOINT(2 2)\rPOINT(3 3)\r\nPOINT(4 4)\nPOINT(5 5) POINT(6 6)\nPOINT(7 7)\tPOINT(8 8)";
        let geometries = read_wkt_geometries(&input[..]);
        let actual: Vec<Geometry<f64>> = geometries.collect();
        let expected = vec![
            Geometry::Point(Point::new(1.0, 1.0)),
            Geometry::Point(Point::new(2.0, 2.0)), // fails to grab point 3 because it's separated by a single \r
            Geometry::Point(Point::new(4.0, 4.0)),
            Geometry::Point(Point::new(5.0, 5.0)), // fails to grab point 6 because it's separated by a space
            Geometry::Point(Point::new(7.0, 7.0)), // fails to grab point 8 because it's separated by a tab
        ];

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_can_parse_3d() {
        let wkt = b"POINT Z(1 2 3)";
        let mut geometries = read_wkt_geometries(&wkt[..]);

        let point = geometries.next();
        assert_eq!(point, Some(Geometry::Point(Point::new(1.0, 2.0))));
    }

    #[test]
    fn test_geometry_and_style_point() {
        let wkt = b"POINT(1 1)";
        let mut geometries = read_wkt_geometries_and_styles(&wkt[..]);
        let point = geometries.next();
        assert_eq!(point, Some(GeometryAndStyle::Point(Point::new(1.0, 1.0))));
    }

    #[test]
    fn test_style_point_radius() {
        let wkt = b"POINTRADIUS(2.0)";
        let mut geometries = read_wkt_geometries_and_styles(&wkt[..]);
        let style = geometries.next();
        assert_eq!(
            style,
            Some(GeometryAndStyle::Style(SvgStyle::PointRadius(2.0)))
        );
    }

    #[test]
    fn test_style_stroke() {
        let wkt = b"STROKE(blue)";
        let mut geometries = read_wkt_geometries_and_styles(&wkt[..]);
        let style = geometries.next();
        assert_eq!(
            style,
            Some(GeometryAndStyle::Style(SvgStyle::Stroke("blue".into())))
        );
    }

    #[test]
    fn test_style_stroke_width() {
        let wkt = b"STROKEWIDTH(2.0)";
        let mut geometries = read_wkt_geometries_and_styles(&wkt[..]);
        let style = geometries.next();
        assert_eq!(
            style,
            Some(GeometryAndStyle::Style(SvgStyle::StrokeWidth(2.0)))
        );
    }

    #[test]
    fn test_style_stroke_dasharray() {
        let wkt = b"STROKEDASHARRAY(1 4)";
        let mut geometries = read_wkt_geometries_and_styles(&wkt[..]);
        let style = geometries.next();
        assert_eq!(
            style,
            Some(GeometryAndStyle::Style(SvgStyle::StrokeDasharray(
                "1 4".into()
            )))
        );
    }

    #[test]
    fn test_style_stroke_dasharray_commas() {
        let wkt = b"STROKEDASHARRAY(1, 4)";
        let mut geometries = read_wkt_geometries_and_styles(&wkt[..]);
        let style = geometries.next();
        assert_eq!(
            style,
            Some(GeometryAndStyle::Style(SvgStyle::StrokeDasharray(
                "1, 4".into()
            )))
        );
    }

    #[test]
    fn test_style_fill() {
        let wkt = b"FILL(red)";
        let mut geometries = read_wkt_geometries_and_styles(&wkt[..]);
        let style = geometries.next();
        assert_eq!(
            style,
            Some(GeometryAndStyle::Style(SvgStyle::Fill("red".into())))
        );
    }
}
