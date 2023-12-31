use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};

use clap::ValueEnum;
use geo::{Geometry, Line, Point};
use petgraph::visit::EdgeRef;
use petgraph::EdgeType;
use wkt::TryFromWkt;

use crate::graph::GeometryGraph;
use crate::io::write_wkt_geometries;

#[derive(Debug, Clone, ValueEnum)]
pub enum GraphFormat {
    /// Output the graph in Trivial Graph Format
    ///
    /// Each node will be labeled with the WKT POINT where it's located.
    Tgf,
    /// Output the geometry graph as pure WKT geometries, one per line.
    Wkt,
}

impl std::fmt::Display for GraphFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            // important: Should match clap::ValueEnum format
            GraphFormat::Wkt => write!(f, "wkt"),
            GraphFormat::Tgf => write!(f, "tgf"),
        }
    }
}

pub fn write_graph<Direction, W>(
    mut writer: W,
    graph: &GeometryGraph<Direction>,
    format: &GraphFormat,
) where
    W: Write,
    Direction: EdgeType,
{
    match format {
        GraphFormat::Tgf => write_tgf_graph(&mut writer, graph),
        GraphFormat::Wkt => write_wkt_graph(writer, graph),
    }
}

pub fn write_tgf_graph<Direction, W>(writer: &mut W, graph: &GeometryGraph<Direction>)
where
    W: Write,
    Direction: EdgeType,
{
    // let (nodes, edges) = graph.into_nodes_edges();
    for idx in graph.node_indices() {
        let coord = graph
            .node_weight(idx)
            .expect("Got index to nonexistent node.");
        let index = idx.index();
        writeln!(writer, "{}\tPOINT({} {})", index, coord.x(), coord.y())
            .expect("Failed to write node label");
    }
    writeln!(writer, "#").expect("Failed to write node/edge separator");
    for edge in graph.edge_references() {
        writeln!(
            writer,
            "{}\t{}",
            edge.source().index(),
            edge.target().index()
        )
        .expect("Failed to write edge");
    }
}

pub fn write_wkt_graph<Direction, W>(writer: W, graph: &GeometryGraph<Direction>)
where
    W: Write,
    Direction: EdgeType,
{
    let edges = graph
        .edge_references()
        .map(|e| Line::new(graph[e.source()], graph[e.target()]))
        .map(Geometry::Line);
    write_wkt_geometries(writer, edges);
}

fn read_raw_node(line: String) -> Result<(usize, Point), String> {
    let mut parts = line.split_whitespace();

    let raw_id = if let Some(id) = parts.next() {
        id
    } else {
        return Err(format!(
            "Failed to parse node: '{line}' missing raw node id",
        ));
    };

    let id = if let Ok(id) = raw_id.parse::<usize>() {
        id
    } else {
        return Err(format!("Failed to parse ID from: '{line}'"));
    };

    let raw_label = parts.fold(String::new(), |a, b| a + b + "\n");
    let label = if let Ok(label) = Point::try_from_wkt_str(&raw_label) {
        label
    } else {
        return Err(format!(
            "Failed to parse node: {id}: label '{raw_label}' as WKT POINT",
        ));
    };

    Ok((id, label))
}

// TODO: Read edge weights
fn read_raw_edge(line: String) -> Result<(usize, usize), String> {
    let mut parts = line.split_whitespace();

    let raw_source = if let Some(source) = parts.next() {
        source
    } else {
        return Err(format!("Failed to parse raw source ID from '{line}'"));
    };
    let raw_target = if let Some(target) = parts.next() {
        target
    } else {
        return Err(format!("Failed to parse raw target ID from '{line}'"));
    };

    let source = if let Ok(source) = raw_source.parse::<usize>() {
        source
    } else {
        return Err(format!(
            "Failed to convert '{raw_source}' to source node ID",
        ));
    };
    let target = if let Ok(target) = raw_target.parse::<usize>() {
        target
    } else {
        return Err(format!(
            "Failed to convert '{raw_target}' to target node ID",
        ));
    };

    Ok((source, target))
}

pub fn read_tgf_graph<Direction, R>(reader: R) -> GeometryGraph<Direction>
where
    R: Read,
    Direction: EdgeType,
{
    let mut lines = BufReader::new(reader).lines();
    let mut raw_to_real = HashMap::new();
    let mut graph = GeometryGraph::<Direction>::default();

    // Read the nodes
    loop {
        let current_line = if let Some(line) = lines.next() {
            line.unwrap()
        } else {
            break;
        };

        if current_line.starts_with('#') {
            break;
        }

        let (raw_id, label) = match read_raw_node(current_line) {
            Ok(node) => node,
            Err(e) => {
                log::warn!("Failed to parse node: {:?}", e);
                continue;
            }
        };

        // There's no guarantee that the nodes were serialized in strictly increasing order, so we
        // have to map between petgraph's node indices and the raw node indices given in the TGF.
        let real_id = graph.add_node(label);
        raw_to_real.insert(raw_id, real_id);
    }

    // Read the edges
    for line in lines {
        let current_line = line.unwrap();
        let (raw_source, raw_target) = match read_raw_edge(current_line) {
            Ok(edge) => edge,
            Err(e) => {
                log::warn!("Failed to parse edge: {:?}", e);
                continue;
            }
        };

        let source = raw_to_real.get(&raw_source);
        let target = raw_to_real.get(&raw_target);
        match (source, target) {
            (Some(real_source), Some(real_target)) => {
                graph.add_edge(*real_source, *real_target, ());
            }
            _ => {
                log::warn!(
                    "Failed to find node from edge {} -> {} in graph",
                    raw_source,
                    raw_target
                );
                continue;
            }
        }
    }

    graph
}

#[cfg(test)]
mod tests {
    use petgraph::{Directed, Undirected};

    use super::*;

    #[test]
    fn test_read_tgf_undirected() {
        let tgf = b"1\tPOINT(0 0)\n42\tPOINT(2 2)\n#\n1\t42";
        let graph = read_tgf_graph::<Undirected, _>(&tgf[..]);

        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.edge_count(), 1);

        let first = graph.node_weight(0.into());
        let expected = Point::new(0.0, 0.0);
        assert_eq!(first, Some(&expected));

        let second = graph.node_weight(1.into());
        let expected = Point::new(2.0, 2.0);
        assert_eq!(second, Some(&expected));

        assert!(graph.contains_edge(0.into(), 1.into()));
        assert!(graph.contains_edge(1.into(), 0.into()));
    }

    #[test]
    fn test_read_tgf_directed() {
        let tgf = b"1\tPOINT(0 0)\n42\tPOINT(2 2)\n#\n1\t42";
        let graph = read_tgf_graph::<Directed, _>(&tgf[..]);

        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.edge_count(), 1);

        let first = graph.node_weight(0.into());
        let expected = Point::new(0.0, 0.0);
        assert_eq!(first, Some(&expected));

        let second = graph.node_weight(1.into());
        let expected = Point::new(2.0, 2.0);
        assert_eq!(second, Some(&expected));

        assert!(graph.contains_edge(0.into(), 1.into()));
        assert!(!graph.contains_edge(1.into(), 0.into()));
    }
}
