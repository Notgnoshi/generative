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

    /// The minimum x coordinate of the grid
    #[clap(short = 'x', long, default_value_t = 0.0)]
    pub min_x: f64,

    /// The maximum x coordinate of the grid
    #[clap(short = 'X', long, default_value_t = 5.0)]
    pub max_x: f64,

    /// The minimum y coordinate of the grid
    #[clap(short = 'y', long, default_value_t = 0.0)]
    pub min_y: f64,

    /// The maximum y coordinate of the grid
    #[clap(short = 'Y', long, default_value_t = 5.0)]
    pub max_y: f64,

    /// The grid spacing. Use --delta-x or --delta-y to specify spacing for the different axes
    #[clap(short = 'd', long)]
    pub delta: Option<f64>,

    /// The x-axis grid spacing.
    #[clap(long)]
    pub delta_x: Option<f64>,

    /// The y-axis grid spacing.
    #[clap(long)]
    pub delta_y: Option<f64>,
}

fn i2x(i: usize, delta_x: f64, min_x: f64) -> f64 {
    (i as f64) * delta_x + min_x
}

fn j2y(j: usize, delta_y: f64, min_y: f64) -> f64 {
    (j as f64) * delta_y + min_y
}

fn x2i(x: f64, delta_x: f64, min_x: f64) -> usize {
    ((x - min_x) / delta_x) as usize
}

fn y2j(y: f64, delta_y: f64, min_y: f64) -> usize {
    ((y - min_y) / delta_y) as usize
}

fn main() {
    let args = CmdlineOptions::parse();

    stderrlog::new()
        .verbosity(args.log_level)
        .color(ColorChoice::Auto)
        .init()
        .expect("Failed to initialize stderrlog");

    let (min_x, min_y) = (args.min_x, args.min_y);
    let (max_x, max_y) = (args.max_x, args.max_y);
    let (mut delta_x, mut delta_y) = if let Some(delta) = args.delta {
        (delta, delta)
    } else {
        (1.0, 1.0)
    };
    if let Some(delta) = args.delta_x {
        delta_x = delta;
    }
    if let Some(delta) = args.delta_y {
        delta_y = delta;
    }
    let width = x2i(max_x - min_x, delta_x, min_x);
    let height = y2j(max_y - min_y, delta_y, min_y);

    let nodes = width * height;
    let edges = 2 * width * height - width - height;
    let mut graph = GeometryGraph::<Undirected>::with_capacity(nodes, edges);

    let max_i = x2i(max_x, delta_x, min_x);
    let max_j = y2j(max_y, delta_y, min_y);

    // Add the nodes
    for j in 0..max_j {
        for i in 0..max_i {
            let x = i2x(i, delta_x, min_x);
            let y = j2y(j, delta_y, min_y);
            let point = Point::new(x, y);
            let _index = graph.add_node(point);
        }
    }

    // Add the edges
    for j in 0..max_j {
        for i in 0..max_i {
            let current_index = width * j + i;

            if i < (width - 1) {
                let east = current_index + 1;
                graph.update_edge(current_index.into(), east.into(), ());
            }
            if j < (height - 1) {
                let north = current_index + width;
                graph.update_edge(current_index.into(), north.into(), ());
            }
            if i > 0 {
                let west = current_index - 1;
                graph.update_edge(current_index.into(), west.into(), ());
            }
            if j > 0 {
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
