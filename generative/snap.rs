use geo::{Coord, CoordsIter, Geometry, Line, LineString, Polygon, Triangle};
use kdtree::KdTree;
use kdtree::distance::squared_euclidean;
use petgraph::EdgeType;
use petgraph::graph::NodeIndex;

use crate::MapCoordsInPlaceMut;
use crate::flatten::{flatten_geometries_into_points_ref, flatten_nested_geometries};
use crate::graph::GeometryGraph;

pub type GeomKdTree = KdTree<f64, Coord, [f64; 2]>;
pub type GraphKdTree = KdTree<f64, NodeIndex<usize>, [f64; 2]>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SnappingStrategy {
    /// Snap to the closest point, modifying the point cloud as the algorithm progresses
    ///
    /// Sensitive to the order in which snapping is performed
    // TODO: There could be lots of possible strategies. During the implementation, I experimented
    // with ClosestInputPoint and ClosestOutputPoint variants, but it got complex enough that I
    // abandoned them. Thinking I could make prune_duplicate_nodes() generic over the different
    // strategies was incorrect.
    ClosestPoint(f64),
    /// Snap points to a regular grid, instead of themselves
    RegularGrid(f64),
}

pub fn snap_geoms(
    geoms: impl Iterator<Item = Geometry>,
    strategy: SnappingStrategy,
) -> Box<dyn Iterator<Item = Geometry>> {
    // Build a k-d tree from the given geometries. flatten_geometries_into_points would require
    // cloning all of the given geometries, so flatten first, and then use the _ref() variant.
    let geoms = flatten_nested_geometries(geoms);
    let geoms: Vec<_> = geoms.collect();

    // Short circuit the creation of the k-d tree
    if let SnappingStrategy::RegularGrid(tolerance) = strategy {
        let snapped = geoms.into_iter().map(move |g| snap_geom_grid(g, tolerance));
        return Box::new(snapped);
    }

    let points = flatten_geometries_into_points_ref(geoms.iter());
    let mut index = GeomKdTree::new(2);
    for point in points {
        let coord: Coord = point.into();
        let coords = [point.x(), point.y()];
        let closest = index.nearest(&coords, 1, &squared_euclidean).unwrap();
        if let Some(closest) = closest.first() {
            let (distance, _) = closest;
            if *distance == 0.0 {
                // This coordinate has already been added
                continue;
            }
        }
        // Since you don't get the coordinates back when doing nearest neighbor lookups, we
        // need to store the coordinates again in the point data.
        index.add(coords, coord).unwrap();
    }

    // TODO: Should this filter out duplicate geometries introduced by snapping? (It already
    // filters duplicate vertices within a geometry, and we can't snap online, so we already have
    // to load all of the geometries into a single vector to begin with)
    let snapped = geoms
        .into_iter()
        .map(move |g| snap_geom(g, &mut index, &strategy));
    Box::new(snapped)
}

pub fn snap_geom(geom: Geometry, index: &mut GeomKdTree, strategy: &SnappingStrategy) -> Geometry {
    match strategy {
        SnappingStrategy::ClosestPoint(tolerance) => snap_geom_impl(geom, index, *tolerance),
        SnappingStrategy::RegularGrid(tolerance) => snap_geom_grid(geom, *tolerance),
    }
}

fn snap_geom_impl(mut geom: Geometry, index: &mut GeomKdTree, tolerance: f64) -> Geometry {
    geom.map_coords_in_place_mut(|c| snap_coord(c, index, tolerance));
    filter_duplicate_vertices(geom)
}

fn snap_coord(coord: Coord, index: &mut GeomKdTree, tolerance: f64) -> Coord {
    // Find the closest two points in the index, because the first closest should always be ourself.
    let coords = [coord.x, coord.y];
    let neighbors = index
        .within(&coords, tolerance, &squared_euclidean)
        .unwrap();
    // We should always find ourselves, or, if move_snapped_point is true, at least find where
    // ourselves have already been snapped to (because one point in the kd-tree could be multiple
    // vertices from multiple geometries).
    debug_assert!(!neighbors.is_empty());

    if !neighbors.is_empty() {
        let (mut _distance, mut found_coords) = neighbors[0];
        // We found ourselves. Now look for a neighbor in range
        if found_coords == &coord && neighbors.len() > 1 {
            // The next closest point
            (_distance, found_coords) = neighbors[1];
        }

        let snapped_coord = *found_coords;
        index.remove(&coords, &coord).unwrap();

        return snapped_coord;
    }

    coord
}

fn snap_coord_grid(coord: Coord, tolerance: f64) -> Coord {
    Coord {
        x: snap_f64_grid(coord.x, tolerance),
        y: snap_f64_grid(coord.y, tolerance),
    }
}

fn snap_f64_grid(value: f64, tolerance: f64) -> f64 {
    // This was waaaay harder than I expected :(
    let rem = value.rem_euclid(tolerance);
    let floor = value - rem;

    // Need to round away from zero, which takes special handling for negative and positive values
    let distance = value - floor;
    let half_tol = 0.5 * tolerance;

    if (value < 0.0 && distance <= half_tol) || distance < half_tol {
        return floor;
    }

    floor + tolerance
}

fn snap_geom_grid(mut geom: Geometry, tolerance: f64) -> Geometry {
    geom.map_coords_in_place_mut(|c| snap_coord_grid(c, tolerance));
    filter_duplicate_vertices(geom)
}

fn filter_duplicate_vertices(geom: Geometry) -> Geometry {
    match geom {
        Geometry::Point(_) => geom,
        Geometry::Line(l) => {
            if l.start == l.end {
                Geometry::Point(l.start.into())
            } else {
                geom
            }
        }
        Geometry::LineString(ls) => filter_duplicate_ls(ls, false),
        Geometry::Polygon(p) => {
            let (exterior, interiors) = p.into_inner();
            let exterior = filter_duplicate_ls(exterior, true);
            match exterior {
                Geometry::Point(_) => exterior,
                Geometry::Line(_) => exterior,
                Geometry::LineString(exterior) => {
                    let mut filtered_interiors = Vec::new();
                    for interior in interiors {
                        if let Geometry::LineString(interior) = filter_duplicate_ls(interior, true)
                        {
                            filtered_interiors.push(interior);
                        }
                    }
                    Geometry::Polygon(Polygon::new(exterior, filtered_interiors))
                }
                _ => unreachable!(),
            }
        }
        Geometry::Rect(r) => filter_duplicate_vertices(Geometry::Polygon(r.to_polygon())),
        Geometry::Triangle(t) => {
            let mut unique = Vec::new();
            for c in t.to_array() {
                if !unique.contains(&c) {
                    unique.push(c);
                }
            }
            if unique.len() == 3 {
                geom
            } else if unique.len() == 2 {
                Geometry::Line(Line::new(unique[0], unique[1]))
            } else {
                Geometry::Point(unique[0].into())
            }
        }
        _ => unreachable!("flatten_nested_geometries in the call graph prevents MULTI-geometries"),
    }
}

fn filter_duplicate_ls(mut ls: LineString, closed: bool) -> Geometry {
    ls.0.dedup();

    if ls.coords_count() == 1 {
        Geometry::Point(ls.0[0].into())
    } else if ls.coords_count() == 2 {
        Geometry::Line(Line::new(ls.0[0], ls.0[1]))
    } else if closed && ls.coords_count() == 3 {
        Geometry::Triangle(Triangle::new(ls.0[0], ls.0[1], ls.0[2]))
    } else {
        Geometry::LineString(ls)
    }
}

pub fn snap_graph<D>(graph: GeometryGraph<D>, strategy: SnappingStrategy) -> GeometryGraph<D>
where
    D: EdgeType,
{
    // You can't look up a node in a graph by its weight (the coordinate)
    // So we need an auxiliary GraphKdTree index for us to look up NodeIndices from their
    // coordinates.
    let mut index = GraphKdTree::new(2);
    for node_idx in graph.node_indices() {
        let node = graph[node_idx];
        let coords = [node.0.x, node.0.y];
        index.add(coords, node_idx).unwrap();
    }

    match strategy {
        SnappingStrategy::ClosestPoint(tolerance) => {
            snap_graph_closest_point(graph, &mut index, tolerance)
        }
        SnappingStrategy::RegularGrid(tolerance) => snap_graph_grid(graph, tolerance),
    }
}

fn snap_graph_closest_point<D>(
    mut graph: GeometryGraph<D>,
    index: &mut GraphKdTree,
    tolerance: f64,
) -> GeometryGraph<D>
where
    D: EdgeType,
{
    let mut nodes_to_remove = Vec::new();
    for node in graph.node_indices() {
        if let Some(snapped) = snap_graph_node(&mut graph, node, index, tolerance) {
            nodes_to_remove.push(snapped);
        }
    }

    // Removing nodes invalidates any existing indices >= the removed index, so remove nodes from
    // greater to smaller, so that smaller indices aren't invalidated by the removal
    nodes_to_remove.sort_unstable();
    for node_idx in nodes_to_remove.into_iter().rev() {
        graph.remove_node(node_idx);
    }

    graph
}

fn snap_graph_node<D>(
    graph: &mut GeometryGraph<D>,
    node_idx: NodeIndex<usize>,
    index: &mut GraphKdTree,
    tolerance: f64,
) -> Option<NodeIndex<usize>>
where
    D: EdgeType,
{
    let coords = [graph[node_idx].0.x, graph[node_idx].0.y];
    let nearest_coords = index
        .within(&coords, tolerance, &squared_euclidean)
        .unwrap();
    debug_assert!(
        !nearest_coords.is_empty(),
        "We'll always look up at least ourselves"
    );

    // There's no node close enough to snap to
    if nearest_coords.len() <= 1 {
        return None;
    }

    let (mut _distance, mut found_idx) = nearest_coords[0];
    let found_coord = graph[*found_idx].0;
    if found_coord == graph[node_idx].0 && nearest_coords.len() > 1 {
        (_distance, found_idx) = nearest_coords[1];
    }
    let found_idx = *found_idx;

    index.remove(&coords, &node_idx).unwrap();

    snap_graph_nodes(graph, node_idx, found_idx);
    Some(node_idx)
}

fn snap_graph_nodes<D>(
    graph: &mut GeometryGraph<D>,
    snap_from: NodeIndex<usize>,
    snap_to: NodeIndex<usize>,
) where
    D: EdgeType,
{
    if snap_from == snap_to || graph[snap_from] == graph[snap_to] {
        return;
    }

    let neighbors: Vec<_> = graph.neighbors(snap_from).collect();
    let mut neighbors_to_snap = Vec::new();
    for neighbor in neighbors {
        // don't add a self-edge
        if neighbor != snap_to {
            graph.update_edge(snap_to, neighbor, ());
            if graph[neighbor].0 == graph[snap_from].0 {
                neighbors_to_snap.push(neighbor);
            }
        }
    }
    graph[snap_from] = graph[snap_to];

    for neighbor in neighbors_to_snap {
        snap_graph_nodes(graph, neighbor, snap_to);
    }
}

fn snap_graph_grid<D>(mut graph: GeometryGraph<D>, tolerance: f64) -> GeometryGraph<D>
where
    D: EdgeType,
{
    let mut index = GraphKdTree::new(2);
    let mut nodes_to_remove = Vec::new();
    for node_idx in graph.node_indices() {
        let snapped_coord = snap_coord_grid(graph[node_idx].0, tolerance);

        // Check if there's already a node that has been snapped to that point, if so, snap the two
        // together in a way that properly adjusts their adjacencies.
        let mut already_snapped = None;
        let snapped_coords = [snapped_coord.x, snapped_coord.y];
        let nearest = index
            .within(&snapped_coords, tolerance / 2.0, &squared_euclidean)
            .unwrap();
        for (_distance, node_idx) in nearest {
            if graph[*node_idx].0 == snapped_coord {
                already_snapped = Some(node_idx);
            }
        }

        if let Some(already_snapped) = already_snapped {
            nodes_to_remove.push(node_idx);
            snap_graph_nodes(&mut graph, node_idx, *already_snapped);
        } else {
            index.add(snapped_coords, node_idx).unwrap();
            graph[node_idx].0 = snapped_coord;
        }
    }

    // Removing nodes invalidates any existing indices >= the removed index, so remove nodes from
    // greater to smaller, so that smaller indices aren't invalidated by the removal
    nodes_to_remove.sort_unstable();
    for node_idx in nodes_to_remove.into_iter().rev() {
        graph.remove_node(node_idx);
    }
    graph
}

#[cfg(test)]
mod tests {
    use std::io::BufWriter;

    use float_cmp::assert_approx_eq;
    use geo::{LineString, Point};
    use petgraph::Undirected;

    use super::*;
    use crate::io::{read_tgf_graph, write_tgf_graph};

    fn get_tgf<D>(graph: &GeometryGraph<D>) -> String
    where
        D: EdgeType,
    {
        let mut buffer = BufWriter::new(Vec::new());
        write_tgf_graph(&mut buffer, graph);
        let buffer = buffer.into_inner().unwrap();
        String::from_utf8_lossy(&buffer).to_string()
    }

    #[test]
    fn test_f64_snapping() {
        let values = [
            // value, tolerance, expected
            (0.0, 1.0, 0.0),
            (0.4, 1.0, 0.0),
            (0.5, 1.0, 1.0),
            (1.0, 1.0, 1.0),
            (-0.0, 1.0, 0.0),
            (-0.4, 1.0, 0.0),
            (-0.5, 1.0, -1.0),
            (-1.5, 1.0, -2.0), // round away from 0.0
            (-1.0, 1.0, -1.0),
            (0.0, 0.5, 0.0),
            (0.24, 0.5, 0.0),
            (0.25, 0.5, 0.5),
            (-0.25, 0.5, -0.5), // round away from 0.0
            (1.4, 0.5, 1.5),
            (1.2, 0.5, 1.0),
            (-1.4, 0.5, -1.5),
            (-1.2, 0.5, -1.0),
        ];
        for (value, tolerance, expected) in values {
            let actual = snap_f64_grid(value, tolerance);
            assert_approx_eq!(f64, actual, expected);
        }
    }

    #[test]
    fn test_coord_grid_snapping() {
        let coord = Coord { x: 1.5, y: 0.5 };
        let expected = Coord { x: 2.0, y: 1.0 };
        let actual = snap_coord_grid(coord, 1.0);
        assert_eq!(actual, expected);

        let coord = Coord { x: 1.4, y: -0.6 };
        let expected = Coord { x: 1.0, y: -1.0 };
        let actual = snap_coord_grid(coord, 1.0);
        assert_eq!(actual, expected);

        let coord = Coord { x: 1.4, y: 0.4 };
        let expected = Coord { x: 1.5, y: 0.5 };
        let actual = snap_coord_grid(coord, 0.5);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_snap_two_points() {
        let geoms = [
            Geometry::Point(Point::new(0.0, 0.0)),
            Geometry::Point(Point::new(0.5, 0.0)),
        ];
        let expected = [
            Geometry::Point(Point::new(0.5, 0.0)),
            // Note this is a duplicate! (geometries aren't deduplicated, but points within a
            // single geometry are).
            Geometry::Point(Point::new(0.5, 0.0)),
        ];

        let actual: Vec<_> =
            snap_geoms(geoms.into_iter(), SnappingStrategy::ClosestPoint(0.6)).collect();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_points_on_linestring_snap_together() {
        let geoms = [Geometry::LineString(LineString::new(vec![
            Coord { x: 0.0, y: 0.0 },
            Coord { x: 0.1, y: 0.0 },
            Coord { x: 1.0, y: 0.0 },
        ]))];
        let expected = [Geometry::Line(Line::new(
            Coord { x: 0.1, y: 0.0 },
            Coord { x: 1.0, y: 0.0 },
        ))];
        let actual: Vec<_> =
            snap_geoms(geoms.into_iter(), SnappingStrategy::ClosestPoint(0.5)).collect();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_snap_graph_grid() {
        let tgf = b"1\tPOINT(0 0)\n42\tPOINT(0.1 0)\n69\tPOINT(2 0)\n#\n1\t42\n42\t69";
        let graph = read_tgf_graph::<Undirected, _>(&tgf[..]);
        assert_eq!(graph.node_count(), 3);
        assert_eq!(graph.edge_count(), 2);

        // The TGF round-trip sanitizes the original node indices
        let tgf = b"0\tPOINT(0 0)\n1\tPOINT(0.2 0)\n2\tPOINT(2 0)\n#\n0\t1\n1\t2\n";
        let expected_tgf = String::from_utf8_lossy(tgf);

        let actual = snap_graph(graph, SnappingStrategy::RegularGrid(0.2));

        // Convert the graph to TGF for human-friendly diffing and output when the tests fail
        let actual_tgf = get_tgf(&actual);
        assert_eq!(actual_tgf, expected_tgf);
    }

    #[test]
    fn test_snap_graph_closest_simple() {
        let tgf = b"0 POINT(0 0)\n1 POINT(1 0)\n2 POINT(1.1 0)\n3 POINT(2 0)\n#\n0 1\n1 2\n2 3";
        let graph = read_tgf_graph::<Undirected, _>(&tgf[..]);
        assert_eq!(graph.node_count(), 4);
        assert_eq!(graph.edge_count(), 3);

        let tgf = b"0\tPOINT(0 0)\n1\tPOINT(2 0)\n2\tPOINT(1.1 0)\n#\n2\t1\n2\t0\n";
        let expected_tgf = String::from_utf8_lossy(tgf);

        let actual = snap_graph(graph, SnappingStrategy::ClosestPoint(0.2));

        let actual_tgf = get_tgf(&actual);
        assert_eq!(actual_tgf, expected_tgf);
    }

    #[test]
    fn test_snap_graph_closest_complex() {
        let tgf = b"0\tPOINT(-0.1 0)\n1\tPOINT(0 0)\n2\tPOINT(0 0.1)\n3\tPOINT(0 -0.1)\n4\tPOINT(2 0)\n#\n0\t1\n2\t1\n3\t1\n1\t4\n";
        let graph = read_tgf_graph::<Undirected, _>(&tgf[..]);

        let tgf = b"0\tPOINT(2 0)\n1\tPOINT(0 -0.1)\n#\n1\t0\n";
        let expected_tgf = String::from_utf8_lossy(tgf);

        let actual = snap_graph(graph, SnappingStrategy::ClosestPoint(0.11));

        let actual_tgf = get_tgf(&actual);
        assert_eq!(actual_tgf, expected_tgf);
    }
}
