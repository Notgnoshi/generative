use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use generative::flatten::flatten_geometries_into_points;
use generative::io::{
    GeometryFormat, GraphFormat, get_input_reader, get_output_writer, read_geometries, write_graph,
};
use generative::triangulation::triangulate;

#[derive(Debug, Clone, ValueEnum)]
enum TriangulationStrategy {
    /// Triangulate each geometry individually
    ///
    /// Meaningful for polygons, not so much for points and linestrings.
    EachGeometry,
    /// Collapse the whole geometry collection into a point cloud to triangulate
    WholeCollection,
}

/// Triangulate the given geometries
#[derive(Debug, Parser)]
#[clap(name = "triangulate", verbatim_doc_comment)]
struct CmdlineOptions {
    /// The log level
    #[clap(short, long, default_value_t = tracing::Level::INFO)]
    log_level: tracing::Level,

    /// Output file to write result to. Defaults to stdout.
    #[clap(short, long)]
    output: Option<PathBuf>,

    /// Output geometry format.
    #[clap(short = 'O', long, default_value_t = GraphFormat::Wkt)]
    output_format: GraphFormat,

    /// Input file to read input from. Defaults to stdin.
    #[clap(short, long)]
    input: Option<PathBuf>,

    /// Input geometry format.
    #[clap(short = 'I', long, default_value_t = GeometryFormat::Wkt)]
    input_format: GeometryFormat,

    /// How to triangulate the input geometries
    #[clap(short, long, default_value = "whole-collection")]
    strategy: TriangulationStrategy,
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
    let mut writer = get_output_writer(&args.output)?;
    let geometries = read_geometries(reader, &args.input_format); // lazily loaded

    match args.strategy {
        TriangulationStrategy::EachGeometry => {
            let triangulations = geometries
                .map(|geom| flatten_geometries_into_points(std::iter::once(geom)))
                .filter_map(triangulate);
            for triangulation in triangulations {
                let graph = triangulation.graph();
                write_graph(&mut writer, &graph, &args.output_format)?;
            }
        }
        TriangulationStrategy::WholeCollection => {
            let points = flatten_geometries_into_points(geometries);
            if let Some(triangulation) = triangulate(points) {
                let graph = triangulation.graph();
                write_graph(writer, &graph, &args.output_format)?;
            }
        }
    }

    Ok(())
}
