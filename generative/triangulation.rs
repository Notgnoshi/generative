/// Calculate the Delaunay triangulation of the given point cloud
pub fn triangulate(points: impl Iterator<Item = geo::Point>) -> Triangulation {
    let points: Vec<delaunator::Point> = points
        .map(|gp| delaunator::Point {
            x: gp.x(),
            y: gp.y(),
        })
        .collect();
    let triangulation = delaunator::triangulate(&points);

    Triangulation {
        points,
        triangulation,
    }
}

pub struct Triangulation {
    points: Vec<delaunator::Point>,
    triangulation: delaunator::Triangulation,
}

impl Triangulation {
    fn triangle_indices(&self) -> impl Iterator<Item = (usize, usize, usize)> + '_ {
        self.triangulation
            .triangles
            .chunks_exact(3)
            .map(|chunk| (chunk[0], chunk[1], chunk[2]))
    }

    fn triangle_points(&self) -> impl Iterator<Item = (geo::Coord, geo::Coord, geo::Coord)> + '_ {
        self.triangle_indices().map(|(a, b, c)| {
            let a = &self.points[a];
            let b = &self.points[b];
            let c = &self.points[c];

            (
                geo::coord! {x: a.x, y: a.y},
                geo::coord! {x: b.x, y: b.y},
                geo::coord! {x: c.x, y: c.y},
            )
        })
    }

    pub fn triangles(&self) -> impl Iterator<Item = geo::Triangle> + '_ {
        self.triangle_points()
            .map(|(a, b, c)| geo::Triangle(a, b, c))
    }

    pub fn lines(&self) -> impl Iterator<Item = geo::Line> + '_ {
        self.triangle_points().flat_map(|(a, b, c)| {
            [
                geo::Line::new(a, b),
                geo::Line::new(b, c),
                geo::Line::new(a, c),
            ]
        })
    }

    pub fn hull(&self) -> geo::Polygon {
        let hull: Vec<geo::Coord> = self
            .triangulation
            .hull
            .iter()
            .map(|i| geo::coord! {x: self.points[*i].x, y: self.points[*i].y})
            .collect();
        let hull = geo::LineString::new(hull);
        geo::Polygon::new(hull, vec![])
    }

    // TODO: Create the graph directly instead of relying on geom2graph?
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::flatten::flatten_geometries_into_points_ref;
    use crate::wkio::read_wkt_geometries;

    #[test]
    fn test_triangulate_unit_square() {
        let wkt = b"POLYGON((0 0, 0 1, 1 1, 1 0, 0 0))";
        let geometries: Vec<_> = read_wkt_geometries(&wkt[..]).collect();
        let points = flatten_geometries_into_points_ref(geometries.iter());

        let triangulation = triangulate(points);

        let triangles: Vec<_> = triangulation.triangles().collect();
        assert_eq!(triangles.len(), 2);
        assert_eq!(
            triangles[0],
            geo::Triangle::new(
                geo::Coord { x: 0., y: 0. },
                geo::Coord { x: 0., y: 1. },
                geo::Coord { x: 1., y: 1. },
            )
        );
        assert_eq!(
            triangles[1],
            geo::Triangle::new(
                geo::Coord { x: 1., y: 1. },
                geo::Coord { x: 1., y: 0. },
                geo::Coord { x: 0., y: 0. },
            )
        );

        assert_eq!(triangulation.lines().count(), 6);

        let wkt = b"POLYGON((1 1, 1 0, 0 0, 0 1, 1 1))"; // same polygon, same winding, different starting point
        let geometries: Vec<_> = read_wkt_geometries(&wkt[..]).collect();
        let hull = triangulation.hull();
        assert_eq!(geo::Geometry::Polygon(hull), geometries[0]);
    }
}
