use std::path::PathBuf;

use clap::Parser;
use generative::flatten::flatten_nested_geometries;
use generative::io::{get_input_reader, get_output_writer, read_geometries, write_geometries};
use geo::{ChaikinSmoothing, Geometry};

/// Smooth the given geometries
#[derive(Debug, Parser)]
#[clap(name = "template", verbatim_doc_comment)]
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

    /// Number of iterations to run Chaikins smoothing algorithm
    #[clap(short = 'n', long, default_value_t = 10)]
    iterations: usize,
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
    let geometries = read_geometries(reader);
    let geometries = flatten_nested_geometries(geometries);
    let geometries = geometries.map(|g| match g {
        Geometry::Point(_)
        | Geometry::Line(_)
        | Geometry::MultiPoint(_)
        | Geometry::GeometryCollection(_)
        | Geometry::Rect(_)
        | Geometry::Triangle(_) => g,
        Geometry::LineString(g) => Geometry::LineString(g.chaikin_smoothing(args.iterations)),
        Geometry::Polygon(g) => Geometry::Polygon(g.chaikin_smoothing(args.iterations)),
        Geometry::MultiLineString(g) => {
            Geometry::MultiLineString(g.chaikin_smoothing(args.iterations))
        }
        Geometry::MultiPolygon(g) => Geometry::MultiPolygon(g.chaikin_smoothing(args.iterations)),
    });

    let writer = get_output_writer(&args.output)?;
    write_geometries(writer, geometries)
}
