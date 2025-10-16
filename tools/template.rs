use std::path::PathBuf;

use clap::Parser;
use generative::io::{
    GeometryFormat, get_input_reader, get_output_writer, read_geometries, write_geometries,
};

/// A template tool
///
/// Useful as a starting point for new tools.
#[derive(Debug, Parser)]
#[clap(name = "template", verbatim_doc_comment)]
struct CmdlineOptions {
    /// The log level
    #[clap(short, long, default_value_t = tracing::Level::INFO)]
    log_level: tracing::Level,

    /// Input file to read input from. Defaults to stdin.
    #[clap(short, long)]
    input: Option<PathBuf>,

    /// Input geometry format.
    #[clap(short = 'I', long, default_value_t = GeometryFormat::Wkt)]
    input_format: GeometryFormat,

    /// Output file to write result to. Defaults to stdout.
    #[clap(short, long)]
    output: Option<PathBuf>,

    /// Output geometry format.
    #[clap(short = 'O', long, default_value_t = GeometryFormat::Wkt)]
    output_format: GeometryFormat,
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
    let geometries = read_geometries(reader, &args.input_format); // lazily loaded

    // Do some kind of transformation to the geometries here.

    let writer = get_output_writer(&args.output)?;
    write_geometries(writer, geometries, args.output_format)
}
