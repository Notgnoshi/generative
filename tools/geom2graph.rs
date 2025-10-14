use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use generative::graph::GeometryGraph;
use generative::io::{
    GeometryFormat, GraphFormat, get_input_reader, get_output_writer, read_geometries,
    read_tgf_graph, write_geometries, write_graph,
};
use generative::noding::{node, polygonize};
use generative::snap::{SnappingStrategy, snap_geoms, snap_graph};
use geo::Geometry;

#[derive(Debug, Clone, ValueEnum)]
enum CliSnappingStrategy {
    ClosestPoint,
    RegularGrid,
}

impl std::fmt::Display for CliSnappingStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            // important: Should match clap::ValueEnum format
            CliSnappingStrategy::ClosestPoint => write!(f, "closest-point"),
            CliSnappingStrategy::RegularGrid => write!(f, "regular-grid"),
        }
    }
}

/// Convert Geometries to Graphs (and back)
#[derive(Debug, Parser)]
#[clap(name = "geom2graph", verbatim_doc_comment)]
struct CmdlineOptions {
    /// The log level
    #[clap(short, long, default_value_t = tracing::Level::INFO)]
    log_level: tracing::Level,

    /// Input file to read input from. Defaults to stdin.
    #[clap(short, long)]
    input: Option<PathBuf>,

    /// Output file to write result to. Defaults to stdout.
    #[clap(short, long)]
    output: Option<PathBuf>,

    /// Format to output --graph2geom geometries as
    #[clap(long, default_value_t = GeometryFormat::Wkt)]
    geometry_format: GeometryFormat,

    /// Format to output (the default) --graph2geom graphs as
    #[clap(long, default_value_t = GraphFormat::Tgf)]
    graph_format: GraphFormat,

    /// Convert the given geometries to a graph. The default
    #[clap(long, conflicts_with = "graph2geom")]
    geom2graph: bool,

    /// Convert the given graph into geometries.
    #[clap(long, conflicts_with = "geom2graph")]
    graph2geom: bool,

    /// Snap together vertices closer than the given tolerance
    #[clap(short, long)]
    tolerance: Option<f64>,

    /// The strategy to use for snapping
    #[clap(long, default_value_t = CliSnappingStrategy::ClosestPoint)]
    snap_strategy: CliSnappingStrategy,
}

fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    let args = CmdlineOptions::parse();

    let filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(args.log_level.into())
        .from_env_lossy();
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_ansi(true)
        .with_writer(std::io::stderr)
        .init();

    let reader = get_input_reader(&args.input)?;
    let writer = get_output_writer(&args.output)?;

    let strategy = match args.snap_strategy {
        CliSnappingStrategy::ClosestPoint => {
            SnappingStrategy::ClosestPoint(args.tolerance.unwrap_or_default())
        }
        CliSnappingStrategy::RegularGrid => {
            SnappingStrategy::RegularGrid(args.tolerance.unwrap_or_default())
        }
    };

    if args.geom2graph || !args.graph2geom {
        let geometries = read_geometries(reader, &args.geometry_format);
        let graph = node::<_, petgraph::Undirected>(geometries);

        let graph = if args.tolerance.is_some() {
            snap_graph(graph, strategy)
        } else {
            graph
        };

        write_graph(writer, &graph, &args.graph_format)
    } else {
        let graph: GeometryGraph<petgraph::Undirected> = read_tgf_graph(reader);
        let (polygons, dangles) = polygonize(&graph);
        let polygons = polygons.into_iter().map(Geometry::Polygon);
        let dangles = dangles.into_iter().map(Geometry::LineString);
        let geometries = polygons.chain(dangles);

        let geometries = if args.tolerance.is_some() {
            snap_geoms(geometries, strategy)
        } else {
            Box::new(geometries)
        };

        write_geometries(writer, geometries, args.geometry_format)
    }
}
