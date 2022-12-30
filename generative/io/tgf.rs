use std::io::Write;

use clap::ValueEnum;
use geo::{Geometry, Line};
use petgraph::visit::EdgeRef;
use petgraph::EdgeType;

use crate::graph::GeometryGraph;
use crate::io::write_wkt_geometries;

#[derive(Debug, Clone, ValueEnum)]
pub enum GraphFormat {
    Tgf,
    Wkt,
}

pub fn write_graph<Direction, W>(writer: W, graph: GeometryGraph<Direction>, format: &GraphFormat)
where
    W: Write,
    Direction: EdgeType,
{
    match format {
        GraphFormat::Tgf => write_graph_tgf(writer, graph),
        GraphFormat::Wkt => write_graph_wkt(writer, graph),
    }
}

fn write_graph_tgf<Direction, W>(mut writer: W, graph: GeometryGraph<Direction>)
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

fn write_graph_wkt<Direction, W>(writer: W, graph: GeometryGraph<Direction>)
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
