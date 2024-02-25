use geo::Geometry;

use crate::cxxbridge;
use crate::graph::GeometryGraph;

pub fn node<G, Direction: petgraph::EdgeType>(geoms: G) -> GeometryGraph<Direction>
where
    G: IntoIterator<Item = Geometry>,
{
    let collection = cxxbridge::GeometryCollectionShim::new(geoms);
    let ffi_graph = unsafe {
        // Setting the tolerance to 0 picks the IteratedNoder instead of the SnappingNoder.
        // They both have pros and cons.
        // * IteratedNoder might throw exceptions if it does not converge on pathological
        //   geometries
        // * SnappingNoder doesn't handle POINT geometries, only LINESTRINGs and POLYGONs
        let tolerance = 0.0;
        cxxbridge::node(&collection, tolerance)
    };
    (&*ffi_graph).into()
}

pub fn polygonize<Direction: petgraph::EdgeType>(graph: &GeometryGraph<Direction>) {
    let ffi_graph = cxxbridge::to_ffi_graph(graph);
}

#[cfg(test)]
mod tests {
    use geo::Point;
    use petgraph::graph::{EdgeIndex, NodeIndex};
    use petgraph::Undirected;

    use super::*;
    use crate::io::read_wkt_geometries;

    #[test]
    fn test_no_geometries() {
        let graph = node::<_, Undirected>(std::iter::empty());
        assert_eq!(graph.node_count(), 0);
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    fn test_points() {
        let wkt = b"POINT(0 0)\nLINESTRING(0 0, 1 1)\nPOINT(2 2)\n";
        let geometries = read_wkt_geometries(&wkt[..]);

        let graph = node::<_, Undirected>(geometries);
        assert_eq!(graph.node_count(), 3);
        assert_eq!(graph.edge_count(), 1);

        let node0 = graph.node_weight(NodeIndex::new(0)).unwrap();
        let expected0 = Point::new(0.0, 0.0);
        assert_eq!(node0, &expected0);

        // Geometries aren't handled one at a time in their order in the iterator. Points are
        // handled, then linestrings, then finally polygons.
        let node1 = graph.node_weight(NodeIndex::new(1)).unwrap();
        let expected1 = Point::new(2.0, 2.0);
        assert_eq!(node1, &expected1);

        let node2 = graph.node_weight(NodeIndex::new(2)).unwrap();
        let expected2 = Point::new(1.0, 1.0);
        assert_eq!(node2, &expected2);

        let edge0 = graph.edge_endpoints(EdgeIndex::new(0)).unwrap();
        let expected0 = (NodeIndex::new(0), NodeIndex::new(2));
        assert_eq!(edge0, expected0);
    }

    #[test]
    fn test_single_linestring() {
        let wkt = b"LINESTRING(0 0, 1 0, 2 0)";
        let geometries = read_wkt_geometries(&wkt[..]);

        let graph = node::<_, Undirected>(geometries);
        assert_eq!(graph.node_count(), 3);
        assert_eq!(graph.edge_count(), 2);

        let node0 = graph.node_weight(NodeIndex::new(0)).unwrap();
        let expected0 = Point::new(0.0, 0.0);
        assert_eq!(node0, &expected0);

        let node1 = graph.node_weight(NodeIndex::new(1)).unwrap();
        let expected1 = Point::new(1.0, 0.0);
        assert_eq!(node1, &expected1);

        let node2 = graph.node_weight(NodeIndex::new(2)).unwrap();
        let expected2 = Point::new(2.0, 0.0);
        assert_eq!(node2, &expected2);

        let edge0 = graph.edge_endpoints(EdgeIndex::new(0)).unwrap();
        let expected0 = (NodeIndex::new(0), NodeIndex::new(1));
        assert_eq!(edge0, expected0);

        let edge1 = graph.edge_endpoints(EdgeIndex::new(1)).unwrap();
        let expected1 = (NodeIndex::new(1), NodeIndex::new(2));
        assert_eq!(edge1, expected1);
    }

    #[test]
    fn test_crossing_linestrings() {
        let wkt = b"LINESTRING(0 0, 1 0)\nLINESTRING(0.5 -1, 0.5 1)";
        let geometries = read_wkt_geometries(&wkt[..]);

        let graph = node::<_, Undirected>(geometries);
        assert_eq!(graph.node_count(), 5);
        assert_eq!(graph.edge_count(), 4);

        let nodes: Vec<_> = graph.node_weights().collect();
        let expected = [
            &Point::new(0.0, 0.0),  // 0
            &Point::new(0.5, 0.0),  // 1 - the new intersection point
            &Point::new(1.0, 0.0),  // 2
            &Point::new(0.5, -1.0), // 3
            &Point::new(0.5, 1.0),  // 4
        ];
        assert_eq!(nodes, expected);

        let mut edges = Vec::new();
        for i in 0..graph.edge_count() {
            let edge = graph.edge_endpoints(EdgeIndex::new(i)).unwrap();
            edges.push((edge.0.index(), edge.1.index()))
        }
        let expected = [(0, 1), (1, 4), (1, 3), (1, 2)];
        assert_eq!(edges, expected);
    }

    #[test]
    fn test_rectangle() {
        // a tic-tac-toe pattern
        let wkt = b"GEOMETRYCOLLECTION( LINESTRING(2 0, 2 8), LINESTRING(6 0, 6 8), LINESTRING(0 2, 8 2), LINESTRING(0 6, 8 6))";
        let geoms = read_wkt_geometries(&wkt[..]);

        let graph = node::<_, Undirected>(geoms);
        assert_eq!(graph.node_count(), 12);
        assert_eq!(graph.edge_count(), 12);

        let nodes: Vec<_> = graph.node_weights().collect();
        let expected = [
            &Point::new(2.0, 0.0), // 0 - start left vertical
            &Point::new(2.0, 2.0), // 1 - intersection
            &Point::new(2.0, 6.0), // 2 - intersection
            &Point::new(2.0, 8.0), // 3 - end left vertical
            &Point::new(6.0, 0.0), // 4 - start right vertical
            &Point::new(6.0, 2.0), // 5 - intersection
            &Point::new(6.0, 6.0), // 6 - intersection
            &Point::new(6.0, 8.0), // 7 - end left vertical
            &Point::new(0.0, 2.0), // 8 - start bottom horizontal
            &Point::new(8.0, 2.0), // 9
            &Point::new(0.0, 6.0), // 10 - start top horizontal
            &Point::new(8.0, 6.0), // 11
        ];
        assert_eq!(nodes, expected);

        let mut edges = Vec::new();
        for i in 0..graph.edge_count() {
            let edge = graph.edge_endpoints(EdgeIndex::new(i)).unwrap();
            edges.push((edge.0.index(), edge.1.index()))
        }
        let expected = [
            (0, 1),  // bottom left vertical dangle
            (1, 5),  // bottom inner horizontal
            (1, 8),  // bottom left horizontal dangle
            (1, 2),  // left inner vertical
            (2, 6),  // top inner horizontal
            (2, 10), // top left horizontal dangle
            (2, 3),  // top left verticle dangle
            (4, 5),  // bottom right verticle dangle
            (5, 9),  // bottom right horizontal dangle
            (5, 6),  // right inner horizontal
            (6, 11), // top right horizontal dangle
            (6, 7),  // top right verticle dangle
        ];
        assert_eq!(edges, expected);
    }
}
