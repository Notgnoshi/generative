use clap::Parser;
use generative::triangulation::GraphFormat;
use generative::wkio::GeometryFormat;
use std::path::PathBuf;

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
