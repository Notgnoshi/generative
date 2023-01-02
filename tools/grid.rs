use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use generative::graph::GeometryGraph;
use generative::io::{
    get_output_writer, write_geometries, write_graph, GeometryFormat, GraphFormat,
};
use geo::{Geometry, Point};
use petgraph::Undirected;
use stderrlog::ColorChoice;

#[derive(Debug, Clone, ValueEnum)]
pub enum GridFormat {
    /// Output the grid as a graph in TGF with WKT POINT node labels
    Graph,
    /// Output the grid lines in WKT
    Lines,
    /// Output the grid points in WKT
    Points,
}

impl std::fmt::Display for GridFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            // important: Should match clap::ValueEnum format
            GridFormat::Graph => write!(f, "graph"),
            GridFormat::Lines => write!(f, "lines"),
            GridFormat::Points => write!(f, "points"),
        }
    }
}
/// Generate a regular grid graph
#[derive(Debug, Parser)]
#[clap(name = "grid", verbatim_doc_comment)]
pub struct CmdlineOptions {
    /// The log level
    #[clap(short, long, default_value_t = log::Level::Info)]
    pub log_level: log::Level,

    /// Output file to write result to. Defaults to stdout.
    #[clap(short, long)]
    pub output: Option<PathBuf>,

    /// Output geometry format.
    #[clap(short = 'O', long, default_value_t = GridFormat::Points)]
    pub output_format: GridFormat,

    /// The dimensions of the grid to generate
    #[clap(number_of_values = 2)]
    pub dimensions: Option<Vec<usize>>,
    // TODO: specify center and spacing?
    // TODO: What about other tilings?
}

fn main() {
    let args = CmdlineOptions::parse();

    stderrlog::new()
        .verbosity(args.log_level)
        .color(ColorChoice::Auto)
        .init()
        .expect("Failed to initialize stderrlog");

    let (width, height) = if let Some(dimensions) = args.dimensions {
        (*dimensions.first().unwrap(), *dimensions.last().unwrap())
    } else {
        (4, 4)
    };
    let nodes = width * height;
    let edges = 2 * width * height - width - height;

    let mut graph = GeometryGraph::<Undirected>::with_capacity(nodes, edges);

    // Add the nodes
    for h in 0..height {
        for w in 0..width {
            let point = Point::new(w as f64, h as f64);
            let index = graph.add_node(point);
            let expected = width * h + w;
            log::trace!(
                "Adding node index: {}, value: POINT({}, {})",
                index.index(),
                w,
                h
            );
            debug_assert_eq!(index.index(), expected);
        }
    }

    // Add the edges
    for h in 0..height {
        for w in 0..width {
            let current_index = width * h + w;

            if w < (width - 1) {
                let east = current_index + 1;
                graph.update_edge(current_index.into(), east.into(), ());
            }
            if h < (height - 1) {
                let north = current_index + width;
                graph.update_edge(current_index.into(), north.into(), ());
            }
            if w > 0 {
                let west = current_index - 1;
                graph.update_edge(current_index.into(), west.into(), ());
            }
            if h > 0 {
                let south = current_index - width;
                graph.update_edge(current_index.into(), south.into(), ());
            }
        }
    }

    let writer = get_output_writer(&args.output).unwrap();
    match args.output_format {
        GridFormat::Graph => write_graph(writer, graph, &GraphFormat::Tgf),
        GridFormat::Lines => write_graph(writer, graph, &GraphFormat::Wkt),
        GridFormat::Points => write_geometries(
            writer,
            graph.node_weights().map(|p| Geometry::Point(*p)),
            &GeometryFormat::Wkt,
        ),
    }
}
