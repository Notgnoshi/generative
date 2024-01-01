use geo::Point;
use petgraph::{Directed, Undirected};

use crate::graph::{GeometryGraph, NodeIndex};

/// Calculate the Delaunay triangulation of the given point cloud
pub fn triangulate(points: impl Iterator<Item = geo::Point>) -> Option<Triangulation> {
    let points: Vec<delaunator::Point> = points
        .map(|gp| delaunator::Point {
            x: gp.x(),
            y: gp.y(),
        })
        .collect();
    if points.len() < 3 {
        return None;
    }
    let triangulation = delaunator::triangulate(&points);

    Some(Triangulation {
        points,
        triangulation,
    })
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

    pub fn digraph(&self) -> GeometryGraph<Directed> {
        let nodes = self.points.len();
        let edges = self.triangulation.halfedges.len();
        let mut graph = GeometryGraph::with_capacity(nodes, edges);

        // Add all the nodes
        for (_i, point) in self.points.iter().enumerate() {
            let point = Point::new(point.x, point.y);
            // NOTE: It's important that the _node_index is the same as the index into the
            // self.points array!
            let _node_index = graph.add_node(point);
            debug_assert_eq!(_node_index.index(), _i);
        }

        // Add the hull edges
        for window in self.triangulation.hull.windows(2) {
            let curr = window[0];
            let next = window[1];

            graph.add_edge(curr.into(), next.into(), ());
        }
        // NOTE: The hull is open and needs to be closed in order to capture the last edge!
        if let (Some(first), Some(last)) = (
            self.triangulation.hull.first(),
            self.triangulation.hull.last(),
        ) {
            graph.add_edge(
                petgraph::graph::NodeIndex::new(*last),
                petgraph::graph::NodeIndex::new(*first),
                (),
            );
        }

        // Add the interior half-edges
        for (dst_i, src_i) in self.triangulation.halfedges.iter().enumerate() {
            // This is a hull edge. The half-edges array doesn't contain enough information to
            // build the graph edge, which is why we loop over the hull above.
            if *src_i == delaunator::EMPTY {
                continue;
            }

            // dst_i and src_i are indices into the triangles array, which itself contains indices
            // into the nodes array.
            let src = self.triangulation.triangles[*src_i];
            let dst = self.triangulation.triangles[dst_i];

            graph.add_edge(src.into(), dst.into(), ());
        }

        graph
    }

    pub fn graph(&self) -> GeometryGraph<Undirected> {
        let digraph = self.digraph();
        let nodes = self.points.len();
        let directed_edges = self.triangulation.halfedges.len();
        let mut graph =
            GeometryGraph::with_capacity(nodes, directed_edges - self.triangulation.hull.len());

        // Add the nodes
        for (_i, node) in digraph.raw_nodes().iter().enumerate() {
            let _node_index = graph.add_node(node.weight);
            debug_assert_eq!(_i, _node_index.index());
        }

        // Add the edges. Use update_edge() to avoid duplicates
        for edge in digraph.raw_edges() {
            #[allow(clippy::unit_arg)]
            let _edge_index = graph.update_edge(edge.source(), edge.target(), edge.weight);
        }

        graph
    }

    #[allow(non_snake_case)]
    fn longest_edge(&self, a: usize, b: usize, c: usize) -> (usize, usize) {
        let A = &self.points[a];
        let B = &self.points[b];
        let C = &self.points[c];

        let mut longest_distance = (A.x - B.x).powi(2) + (A.y - B.y).powi(2);
        let mut longest = (a, b);

        let distance = (B.x - C.x).powi(2) + (B.y - C.y).powi(2);
        if distance > longest_distance {
            longest_distance = distance;
            longest = (b, c);
        }

        let distance = (A.x - C.x).powi(2) + (A.y - C.y).powi(2);
        if distance > longest_distance {
            longest = (a, c);
        }

        longest
    }

    pub fn urquhart(&self) -> GeometryGraph<Undirected> {
        let mut graph = self.graph();

        // According to https://en.wikipedia.org/wiki/Urquhart_graph you can construct the Urquhart
        // graph by removing the longest edge of each triangle in the Delaunay triangulation.

        for (a, b, c) in self.triangle_indices() {
            let (src, dst) = self.longest_edge(a, b, c);

            let src_index = petgraph::graph::NodeIndex::<NodeIndex>::new(src);
            let dst_index = petgraph::graph::NodeIndex::<NodeIndex>::new(dst);
            if let Some(edge_index) = graph.find_edge(src_index, dst_index) {
                // It's not very efficient to remove one edge at a time, but that's the best I can
                // do with the graph API
                graph.remove_edge(edge_index);
            }
        }

        graph
    }
}

#[cfg(test)]
mod tests {
    use delaunator::EMPTY;

    use super::*;
    use crate::flatten::flatten_geometries_into_points_ref;
    use crate::io::read_wkt_geometries;

    #[test]
    fn test_triangulate_unit_square() {
        let wkt = b"POLYGON((0 0, 0 1, 1 1, 1 0, 0 0))";
        let geometries: Vec<_> = read_wkt_geometries(&wkt[..]).collect();
        let points = flatten_geometries_into_points_ref(geometries.iter());

        let triangulation = triangulate(points).unwrap();

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

    #[test]
    fn test_graph() {
        let wkt = b"POINT (65.85186826230156 -39.36525618186133)\n\
                    POINT (61.35756898870892 -34.85194590696902)\n\
                    POINT (38.25507241174806 -24.06029365638358)\n\
                    POINT (9.25896849065506 -63.356505724778266)\n\
                    POINT (-5.692741678486288 -17.181741298068346)\n\
                    POINT (-32.93567551272198 -61.274655097506745)\n";
        let geometries: Vec<_> = read_wkt_geometries(&wkt[..]).collect();
        assert_eq!(geometries.len(), 6);
        let points = flatten_geometries_into_points_ref(geometries.iter());
        let triangulation = triangulate(points).unwrap();

        // NOTE: This all makes much more sense if you draw a picture! Pipe the following through
        // render.py:
        //      cargo test --all-features test_graph -- --nocapture | ./tools/render.py
        #[cfg(feature = "test-io")]
        {
            let lines = triangulation.lines().map(geo::Geometry::Line);
            write_wkt_geometries(std::io::stdout(), geometries);
            write_wkt_geometries(std::io::stdout(), lines);
        }

        let triangles = [3, 5, 4, 4, 2, 3, 2, 1, 3, 1, 0, 3];
        assert_eq!(triangulation.triangulation.triangles, triangles);
        let halfedges = [EMPTY, EMPTY, 5, EMPTY, 8, 2, EMPTY, 11, 4, EMPTY, EMPTY, 7];
        assert_eq!(triangulation.triangulation.halfedges, halfedges);

        // NOTE: None of the indices from 'triangles' or 'halfedges' index into 'hull'! You have to
        // create the hull half-edges by looping over the hull. However, be aware that the hull is
        // an open LINESTRING, and needs to be implicitly closed in order to capture the last edge!

        let hull = [1, 0, 3, 5, 4, 2];
        assert_eq!(triangulation.triangulation.hull, hull);

        let graph = triangulation.digraph();
        // It's not necessary that the edges be compared in order, but that's easiest to implement
        // here.
        let edges: Vec<_> = graph
            .raw_edges()
            .iter()
            .map(|e| (e.source().index(), e.target().index()))
            .collect();
        let expected = vec![
            (1, 0),
            (0, 3),
            (3, 5),
            (5, 4),
            (4, 2),
            (2, 1),
            (3, 4),
            (3, 2),
            (4, 3),
            (3, 1),
            (2, 3),
            (1, 3),
        ];
        assert_eq!(edges, expected);

        let graph = triangulation.graph();
        let edges: Vec<_> = graph
            .raw_edges()
            .iter()
            .map(|e| (e.source().index(), e.target().index()))
            .collect();
        let expected = vec![
            (1, 0),
            (0, 3),
            (3, 5),
            (5, 4),
            (4, 2),
            (2, 1),
            (3, 4),
            (3, 2),
            (3, 1),
        ];
        assert_eq!(edges, expected);
    }

    #[test]
    fn test_urquhart() {
        let wkt = b"POINT (65.85186826230156 -39.36525618186133)\n\
                    POINT (61.35756898870892 -34.85194590696902)\n\
                    POINT (38.25507241174806 -24.06029365638358)\n\
                    POINT (9.25896849065506 -63.356505724778266)\n\
                    POINT (-5.692741678486288 -17.181741298068346)\n\
                    POINT (-32.93567551272198 -61.274655097506745)\n";
        let geometries: Vec<_> = read_wkt_geometries(&wkt[..]).collect();
        assert_eq!(geometries.len(), 6);
        let points = flatten_geometries_into_points_ref(geometries.iter());
        let triangulation = triangulate(points).unwrap();

        let graph = triangulation.graph();
        let edges: Vec<_> = graph
            .raw_edges()
            .iter()
            .map(|e| (e.source().index(), e.target().index()))
            .collect();
        let expected = vec![
            (1, 0),
            (0, 3),
            (3, 5),
            (5, 4),
            (4, 2),
            (2, 1),
            (3, 4),
            (3, 2),
            (3, 1),
        ];
        assert_eq!(edges, expected);

        let graph = triangulation.urquhart();
        let edges: Vec<_> = graph
            .raw_edges()
            .iter()
            .map(|e| (e.source().index(), e.target().index()))
            .collect();
        let expected = vec![(1, 0), (2, 1), (3, 5), (3, 4), (4, 2)];
        assert_eq!(edges, expected);

        // NOTE: This all makes much more sense if you draw a picture! Pipe the following through
        // render.py:
        //      cargo test --all-features test_urquhart -- --nocapture | ./tools/render.py
        //
        #[cfg(feature = "test-io")]
        {
            let points: Vec<_> = triangulation
                .points
                .iter()
                .map(|p| geo::coord! {x: p.x, y: p.y})
                .collect();
            let lines = edges
                .iter()
                .map(|(a, b)| geo::Line::new(points[*a], points[*b]))
                .map(geo::Geometry::Line);
            write_wkt_geometries(std::io::stdout(), lines);
        }
    }
}
