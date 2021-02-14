use log::trace;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
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
    input: Option<PathBuf>,

    /// Output file to write to. Defaults to stdout.
    #[structopt(short, long, parse(from_os_str))]
    output: Option<PathBuf>,
}

impl Options {
    // Redefine StructOpt::from_args so that it's visible in main() without having to use StructOpt
    // This is because trait methods are scoped with the trait.
    pub fn from_args() -> Options {
        <Options as StructOpt>::from_args()
    }
    /// Get a BufWriter for the given path or stdout.
    pub fn get_output_writer(&self) -> BufWriter<Box<dyn Write>> {
        match &self.output {
            Some(path) => match File::create(&path) {
                Err(why) => panic!("couldn't create {} because: {}", path.display(), why),
                Ok(file) => {
                    trace!("Writing to {}", path.display());
                    BufWriter::new(Box::new(file))
                }
            },
            None => {
                trace!("Writing to stdout");
                BufWriter::new(Box::new(std::io::stdout()))
            }
        }
    }

    /// Get a BufReader for the given path or stdin.
    pub fn get_input_reader(&self) -> BufReader<Box<dyn Read>> {
        match &self.input {
            Some(path) => match File::open(&path) {
                Err(why) => panic!("couldn't read {} because: {}", path.display(), why),
                Ok(file) => {
                    trace!("Reading from {}", path.display());
                    BufReader::new(Box::new(file))
                }
            },
            None => {
                trace!("Reading from stdin");
                BufReader::new(Box::new(std::io::stdin()))
            }
        }
    }
}
