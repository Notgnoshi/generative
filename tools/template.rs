use std::path::PathBuf;

use clap::Parser;
use generative::io::{
    get_input_reader, get_output_writer, read_geometries, write_geometries, GeometryFormat,
};
use stderrlog::ColorChoice;

/// A template tool
///
/// Useful as a starting point for new tools.
#[derive(Debug, Parser)]
#[clap(name = "template", verbatim_doc_comment)]
struct CmdlineOptions {
    /// The log level
    #[clap(short, long, default_value_t = log::Level::Info)]
    log_level: log::Level,

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

fn main() {
    let args = CmdlineOptions::parse();

    stderrlog::new()
        .verbosity(args.log_level)
        .color(ColorChoice::Auto)
        .init()
        .expect("Failed to initialize stderrlog");

    let reader = get_input_reader(&args.input).unwrap();
    let geometries = read_geometries(reader, &args.input_format); // lazily loaded

    // Do some kind of transformation to the geometries here.

    let writer = get_output_writer(&args.output).unwrap();
    write_geometries(writer, geometries, args.output_format);
}
