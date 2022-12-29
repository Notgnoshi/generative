use geo::Geometry;

struct GeometryFlattener<G>
where
    G: Iterator<Item = Geometry>,
{
    // The geometries to flatten
    geometries: G,

    // An iterator that will consume the current geometry. In the case of a GeometryCollection,
    // this will be a recursive GeometryFlattener!
    current_geometry_iterator: Box<dyn Iterator<Item = Geometry>>,
}

impl<G> Iterator for GeometryFlattener<G>
where
    G: Iterator<Item = Geometry>,
{
    type Item = Geometry;

    fn next(&mut self) -> Option<Self::Item> {
        let current_geometry = self.current_geometry_iterator.next();
        if current_geometry.is_some() {
            return current_geometry;
        }
        // We've exhausted the iterator for the current geometry, move on to the next one
        let next_geometry = self.geometries.next();
        self.current_geometry_iterator = geometry_to_outer_iterator(next_geometry);

        self.current_geometry_iterator.next()
    }
}

fn geometry_to_outer_iterator(geometry: Option<Geometry>) -> Box<dyn Iterator<Item = Geometry>> {
    if let Some(geometry) = geometry {
        match geometry {
            Geometry::Point(_)
            | Geometry::Line(_)
            | Geometry::LineString(_)
            | Geometry::Polygon(_)
            | Geometry::Rect(_)
            | Geometry::Triangle(_) => Box::new(std::iter::once(geometry)),
            Geometry::MultiPoint(mp) => Box::new(mp.0.into_iter().map(Geometry::Point)),
            Geometry::MultiLineString(mls) => Box::new(mls.0.into_iter().map(Geometry::LineString)),
            Geometry::MultiPolygon(mp) => Box::new(mp.0.into_iter().map(Geometry::Polygon)),
            // This is the important case to consider, because we have to resort to recursion,
            // because GEOMETRYCOLLECTIONs can be arbitrarily nested.
            Geometry::GeometryCollection(gc) => Box::new(flatten_nested_geometries(gc.0)),
        }
    } else {
        Box::new(std::iter::empty())
    }
}

// Compare this against the C++ GeometryFlattener and weep for joy.
pub fn flatten_nested_geometries<G>(geometries: G) -> impl Iterator<Item = Geometry>
where
    G: IntoIterator<Item = Geometry>,
{
    let mut geometries = geometries.into_iter();
    let current_geometry = geometries.next();
    let current_geometry_iterator = geometry_to_outer_iterator(current_geometry);
    GeometryFlattener {
        geometries,
        current_geometry_iterator,
    }
}

// TODO: This is seemingly impossible, because you can't create a &Geometry::Point from a &Point
// (and the same for the other concrete geometry types).
//
// pub fn flatten_nested_geometry_refs<'g, G>(geometries: G) -> impl Iterator<Item = &'g Geometry>
// where
//     G: IntoIterator<Item = &'g Geometry>,
// {
//     ...
// }

#[cfg(test)]
mod tests {
    use geo::Point;

    use super::*;
    use crate::wkio::read_wkt_geometries;

    #[test]
    fn test_flatten_single_point() {
        let inputs = std::iter::once(Geometry::Point(Point::new(1.0, 1.0)));
        let flattened: Vec<_> = flatten_nested_geometries(inputs).collect();
        let expected = [Geometry::Point(Point::new(1.0, 1.0))];
        assert_eq!(flattened, expected);
    }

    #[test]
    fn test_flatten_single_multi_point() {
        let points = vec![
            Point::new(1.0, 1.0),
            Point::new(2.0, 2.0),
            Point::new(3.0, 3.0),
        ];
        let multipoint = Geometry::MultiPoint(geo::MultiPoint::new(points.clone()));
        let multipoint = std::iter::once(multipoint);

        let flattened: Vec<_> = flatten_nested_geometries(multipoint).collect();

        let expected: Vec<_> = points.into_iter().map(Geometry::Point).collect();
        assert_eq!(flattened, expected);
    }

    #[test]
    fn test_flatten_two_multi_points() {
        let mut points1 = vec![
            Point::new(1.0, 1.0),
            Point::new(2.0, 2.0),
            Point::new(3.0, 3.0),
        ];
        let multipoint1 = Geometry::MultiPoint(geo::MultiPoint::new(points1.clone()));
        let mut points2 = vec![
            Point::new(4.0, 4.0),
            Point::new(5.0, 5.0),
            Point::new(6.0, 6.0),
        ];
        let multipoint2 = Geometry::MultiPoint(geo::MultiPoint::new(points2.clone()));
        let multipoints = [multipoint1, multipoint2];

        let flattened: Vec<_> = flatten_nested_geometries(multipoints).collect();

        points1.append(&mut points2);
        let expected: Vec<_> = points1.into_iter().map(Geometry::Point).collect();

        assert_eq!(flattened, expected);
    }

    #[test]
    fn test_flatten_simple_collection() {
        let wkt = b"GEOMETRYCOLLECTION(POINT(1 1))";
        let geometries = read_wkt_geometries(&wkt[..]);
        let flattened: Vec<_> = flatten_nested_geometries(geometries).collect();
        let expected = [Geometry::Point(Point::new(1.0, 1.0))];
        assert_eq!(flattened, expected);
    }

    #[test]
    fn test_flatten_collection_with_multi() {
        let wkt = b"GEOMETRYCOLLECTION(MULTIPOINT(1 1, 2 2), POINT(3 3))";
        let geometries = read_wkt_geometries(&wkt[..]);
        let flattened: Vec<_> = flatten_nested_geometries(geometries).collect();
        let expected = [
            Point::new(1.0, 1.0).into(),
            Point::new(2.0, 2.0).into(),
            Point::new(3.0, 3.0).into(),
        ];
        assert_eq!(flattened, expected);
    }

    #[test]
    fn test_flatten_nested_collections() {
        let wkt = b"GEOMETRYCOLLECTION(POINT(1 1), MULTIPOINT(2 2), GEOMETRYCOLLECTION(POINT(0 0)), GEOMETRYCOLLECTION(MULTIPOINT(3 3, 4 4)))";
        let geometries: Vec<_> = read_wkt_geometries(&wkt[..]).collect();
        assert_eq!(geometries.len(), 1);
        let flattened: Vec<_> = flatten_nested_geometries(geometries).collect();
        let expected = [
            Point::new(1.0, 1.0).into(),
            Point::new(2.0, 2.0).into(),
            Point::new(0.0, 0.0).into(),
            Point::new(3.0, 3.0).into(),
            Point::new(4.0, 4.0).into(),
        ];
        assert_eq!(flattened, expected);
    }
}
