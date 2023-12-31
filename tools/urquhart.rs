use std::path::PathBuf;

use clap::Parser;
use generative::flatten::flatten_geometries_into_points;
use generative::io::{
    get_input_reader, get_output_writer, read_geometries, write_graph, GeometryFormat, GraphFormat,
};
use generative::triangulation::triangulate;
use stderrlog::ColorChoice;

/// Generate the Urquhart graph of the given geometries
///
/// Approximates the point cloud's relative neighborhood.
#[derive(Debug, Parser)]
#[clap(name = "urquhart", verbatim_doc_comment)]
pub struct CmdlineOptions {
    /// The log level
    #[clap(short, long, default_value_t = log::Level::Info)]
    pub log_level: log::Level,

    /// Output file to write result to. Defaults to stdout.
    #[clap(short, long)]
    pub output: Option<PathBuf>,

    /// Output geometry format.
    #[clap(short = 'O', long, default_value_t = GraphFormat::Wkt)]
    pub output_format: GraphFormat,

    /// Input file to read input from. Defaults to stdin.
    #[clap(short, long)]
    pub input: Option<PathBuf>,

    /// Input geometry format.
    #[clap(short = 'I', long, default_value_t = GeometryFormat::Wkt)]
    pub input_format: GeometryFormat,
}

fn main() {
    let args = CmdlineOptions::parse();

    stderrlog::new()
        .verbosity(args.log_level)
        .color(ColorChoice::Auto)
        .init()
        .expect("Failed to initialize stderrlog");

    let reader = get_input_reader(&args.input).unwrap();
    let geometries = read_geometries(reader, &args.input_format); // lazily loaded

    let points = flatten_geometries_into_points(geometries);
    if let Some(triangulation) = triangulate(points) {
        let urquhart = triangulation.urquhart();

        let writer = get_output_writer(&args.output).unwrap();
        write_graph(writer, &urquhart, &args.output_format);
    }
}
