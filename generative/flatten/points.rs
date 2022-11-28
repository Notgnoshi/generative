use geo::{CoordsIter, Geometry, Point};

/// Flatten the given geometries into a point cloud
///
/// NOTE: Polygons are closed - meaning that you'll always get a 'duplicate' start and end point
/// for each ring.
pub fn flatten_geometries_into_points(
    geometries: impl Iterator<Item = Geometry<f64>>,
) -> impl Iterator<Item = Point<f64>> {
    geometries
        .flat_map(|geom| {
            // TODO: Consider implicitly opening each ring. Would need to recursively flatten
            // MULTI-geometries first though, so that I wouldn't have to sort and remove duplicates
            // after the fact.

            // It's ridiculous that I have to collect() here. It's not actually needless. The geom
            // is dropped when the closure returns, so we need an iterator that moves the geom,
            // rather than coords_iter(), which just borrows it.
            #[allow(clippy::needless_collect)]
            let coords: Vec<geo::Coord<f64>> = geom.coords_iter().collect();
            coords.into_iter()
        })
        .map(|coord| coord.into())
}

/// A variant of [`flatten_geometries_into_points`](flatten_geometries_into_points) that doesn't
/// consume the geometries
pub fn flatten_geometries_into_points_ref<'geom>(
    geometries: impl Iterator<Item = &'geom Geometry<f64>> + 'geom,
) -> impl Iterator<Item = Point<f64>> + 'geom {
    geometries
        .flat_map(|geom| geom.coords_iter())
        .map(|coord| coord.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wkio::read_wkt_geometries;

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
        assert_eq!(points.len(), 10);

        let expected = vec![
            Point::new(0.0, 0.0),
            Point::new(1.0, 0.0),
            Point::new(1.0, 1.0),
            Point::new(0.0, 1.0),
            Point::new(0.0, 0.0),
            Point::new(0.25, 0.25),
            Point::new(0.75, 0.25),
            Point::new(0.75, 0.75),
            Point::new(0.25, 0.75),
            Point::new(0.25, 0.25),
        ];
        assert_eq!(points, expected);
    }
}
