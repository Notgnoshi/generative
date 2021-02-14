use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "geom2graph",
    about = "A CLI application to convert geometries into a graph data structure."
)]
pub struct Options {
    /// Silence all logging.
    #[structopt(short, long)]
    pub quiet: bool,

    /// Increase output verbosity.
    #[structopt(short, long, parse(from_occurrences))]
    pub verbose: usize,

    /// Input file to read from. Defaults to stdin.
    #[structopt(short, long, parse(from_os_str))]
    pub input: Option<PathBuf>,

    /// Output file to write to. Defaults to stdout.
    #[structopt(short, long, parse(from_os_str))]
    pub output: Option<PathBuf>,
}

impl Options {
    // Redefine StructOpt::from_args so that it's visible in main() without having to use StructOpt
    // This is because trait methods are scoped with the trait.
    pub fn from_args() -> Options {
        <Options as StructOpt>::from_args()
    }
}
