use geo::{Coord, CoordsIter, Geometry, LineString, Point};

use crate::flatten::flatten_nested_geometries;

fn implicitly_open_linestring(ls: LineString) -> impl Iterator<Item = Coord> {
    let length = if ls.is_closed() {
        ls.0.len() - 1
    } else {
        ls.0.len()
    };
    ls.0.into_iter().take(length)
}

fn implicitly_open(geometry: Geometry) -> Box<dyn Iterator<Item = Coord>> {
    match geometry {
        Geometry::Point(_) | Geometry::Line(_) | Geometry::Rect(_) | Geometry::Triangle(_) => {
            // This collect _is_ actually needed :(
            //
            // .coords_iter() doesn't consume the geometry, but the function signature for
            // implicitly_open() requires that we do, so that's what we'll do then.
            #[allow(clippy::needless_collect)]
            let coords: Vec<_> = geometry.coords_iter().collect();
            Box::new(coords.into_iter())
        }
        Geometry::LineString(ls) => Box::new(implicitly_open_linestring(ls)),
        Geometry::Polygon(p) => {
            let (exterior, interiors) = p.into_inner();

            let exterior = implicitly_open_linestring(exterior);
            let interiors = interiors.into_iter().flat_map(implicitly_open_linestring);
            let all_points = exterior.chain(interiors);
            Box::new(all_points)
        }
        _ => unreachable!("MULTI-geometries are flattened in the caller of this function"),
    }
}

fn implicitly_open_ref(geometry: &Geometry) -> Box<dyn Iterator<Item = Coord> + '_> {
    match geometry {
        Geometry::Point(_) | Geometry::Line(_) | Geometry::Rect(_) | Geometry::Triangle(_) => {
            Box::new(geometry.coords_iter())
        }
        Geometry::LineString(ls) => {
            let ls = ls.clone();
            Box::new(implicitly_open_linestring(ls))
        }
        Geometry::Polygon(p) => {
            let p = p.clone();
            let (exterior, interiors) = p.into_inner();

            let exterior = implicitly_open_linestring(exterior);
            let interiors = interiors.into_iter().flat_map(implicitly_open_linestring);
            let all_points = exterior.chain(interiors);
            Box::new(all_points)
        }
        // TODO: Is there an alternative implementation of flatten_nested_geometries that could
        // operate on an 'impl Iterator<Item = &Geometry>'?
        _ => unimplemented!(
            "You can't flatten multi-geometries into point clouds without consuming the geometry"
        ),
    }
}

/// Flatten the given geometries into a point cloud
///
/// NOTE: Closed rings are implicitly opened.
pub fn flatten_geometries_into_points(
    geometries: impl Iterator<Item = Geometry>,
) -> impl Iterator<Item = Point> {
    let geometries = flatten_nested_geometries(geometries);

    geometries
        .flat_map(implicitly_open)
        .map(|coord| coord.into())
}

/// A variant of [`flatten_geometries_into_points`] that doesn't consume the geometries
///
/// NOTE: Closed rings are implicitly opened.
pub fn flatten_geometries_into_points_ref<'geom>(
    geometries: impl Iterator<Item = &'geom Geometry> + 'geom,
) -> impl Iterator<Item = Point> + 'geom {
    geometries
        .flat_map(implicitly_open_ref)
        .map(|coord| coord.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::read_wkt_geometries;

    #[test]
    fn test_flatten_single_point() {
        let wkt = b"POINT(1 1)";
        let geometries = read_wkt_geometries(&wkt[..]);
        let mut points = flatten_geometries_into_points(geometries);
        assert_eq!(points.next().unwrap(), Point::new(1.0, 1.0));

        let geometries = read_wkt_geometries(&wkt[..]);
        let geometries: Vec<Geometry<f64>> = geometries.collect();
        assert_eq!(geometries.len(), 1);

        // Don't consume the geometries from the array
        {
            let mut points = flatten_geometries_into_points_ref(geometries.iter());
            assert_eq!(points.next().unwrap(), Point::new(1.0, 1.0));

            // Need to drop 'points' here so that we're not borrowing the 'geometries' array when
            // we call into_iter() later.
        }

        // Consume the geometries from the array
        let mut points = flatten_geometries_into_points(geometries.into_iter());
        assert_eq!(points.next().unwrap(), Point::new(1.0, 1.0));

        // can't do anything with geometries here.
        // assert_eq!(geometries.len(), 1);
    }

    #[test]
    fn test_flatten_polygon() {
        let wkt = b"POLYGON((0 0, 1 0, 1 1, 0 1, 0 0), (0.25 0.25, 0.75 0.25, 0.75 0.75, 0.25 0.75, 0.25 0.25))";
        let geometries = read_wkt_geometries(&wkt[..]);
        let points: Vec<Point<f64>> = flatten_geometries_into_points(geometries).collect();
        assert_eq!(points.len(), 10 - 2);

        let expected = vec![
            Point::new(0.0, 0.0),
            Point::new(1.0, 0.0),
            Point::new(1.0, 1.0),
            Point::new(0.0, 1.0),
            // Point::new(0.0, 0.0),
            Point::new(0.25, 0.25),
            Point::new(0.75, 0.25),
            Point::new(0.75, 0.75),
            Point::new(0.25, 0.75),
            // Point::new(0.25, 0.25),
        ];
        assert_eq!(points, expected);
    }
}
