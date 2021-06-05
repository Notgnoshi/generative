use crate::dla;
use petgraph::visit::EdgeRef;
use std::io::{BufWriter, Write};

pub fn format_tgf(writer: &mut BufWriter<Box<dyn Write>>, graph: dla::GraphType) {
    // let (nodes, edges) = graph.into_nodes_edges();
    for idx in graph.node_indices() {
        let particle = graph
            .node_weight(idx)
            .expect("Got index to nonexistent node.");
        let label = idx.index();
        writeln!(
            writer,
            "{}\tPOINT({} {})",
            label, particle.coordinates[0], particle.coordinates[1]
        )
        .expect("Failed to write node label");
    }
    writeln!(writer, "#").expect("Failed to write node/edge separator");
    for edge in graph.edge_references() {
        writeln!(
            writer,
            "{}\t {}",
            edge.source().index(),
            edge.target().index()
        )
        .expect("Failed to write edge");
    }
}

pub fn format_wkt(writer: &mut BufWriter<Box<dyn Write>>, graph: dla::GraphType) {
    for idx in graph.node_indices() {
        let particle = graph
            .node_weight(idx)
            .expect("Got index to nonexistent node.");
        writeln!(
            writer,
            "POINT ({} {})",
            particle.coordinates[0], particle.coordinates[1]
        )
        .expect("Failed to write node WKT.");
    }
}
