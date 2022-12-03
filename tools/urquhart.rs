use clap::Parser;
use generative::flatten::flatten_geometries_into_points;
use generative::stdio::{get_input_reader, get_output_writer};
use generative::triangulation::GraphFormat;
use generative::triangulation::{triangulate, write_graph};
use generative::wkio::{read_geometries, GeometryFormat};
use std::path::PathBuf;
use stderrlog::ColorChoice;

/// Generate the Urquhart graph of the given geometries
///
/// Approximates the point cloud's relative neighborhood.
#[derive(Debug, Parser)]
#[clap(name = "urquhart", verbatim_doc_comment)]
pub struct CmdlineOptions {
    /// Increase logging verbosity. Defaults to ERROR level.
    #[clap(short, long, action = clap::ArgAction::Count)]
    pub verbosity: u8,

    /// Output file to write result to. Defaults to stdout.
    #[clap(short, long)]
    pub output: Option<PathBuf>,

    /// Output geometry format.
    #[clap(short = 'O', long, default_value = "wkt")]
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
        .verbosity(args.verbosity as usize + 1) // Default to WARN level.
        .color(ColorChoice::Auto)
        .init()
        .expect("Failed to initialize stderrlog");

    let reader = get_input_reader(&args.input).unwrap();
    let geometries = read_geometries(reader, &args.input_format); // lazily loaded

    let points = flatten_geometries_into_points(geometries);
    let triangulation = triangulate(points);
    let urquhart = triangulation.urquhart();

    let writer = get_output_writer(&args.output).unwrap();
    write_graph(writer, urquhart, &args.output_format);
}