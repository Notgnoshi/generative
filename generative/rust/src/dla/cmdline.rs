use log::trace;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::str::FromStr;
use std::string::ParseError;
use structopt::StructOpt;

/// Specifies the plaintext output format.
/// In all cases, the seed points will be written first.
#[derive(Debug)]
pub enum OutputFormat {
    /// Use TGF graph output format, labelling each node with the WKT point.
    /// Edges will be unlabeled with weight, as that can be done in a post-processing
    /// step, which simplifies the datastructure here.
    GraphTGF,
    /// Prints each point in WKT on its own line.
    PointCloudWKT,
}

impl FromStr for OutputFormat {
    type Err = ParseError;
    fn from_str(format: &str) -> Result<Self, Self::Err> {
        match format {
            "tgf" => Ok(OutputFormat::GraphTGF),
            "points" => Ok(OutputFormat::PointCloudWKT),
            _ => {
                // Logging isn't initialized yet.
                // error!(
                //     "Failed to parse {} output format. Falling back to \"tgf\".",
                //     format
                // );
                Ok(OutputFormat::GraphTGF)
            }
        }
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "dla", about = env!("CARGO_PKG_DESCRIPTION"))]
pub struct Options {
    /// Silence all logging
    #[structopt(short, long)]
    pub quiet: bool,

    /// Increase logging verbosity.
    #[structopt(short, long, parse(from_occurrences))]
    pub verbose: usize,

    /// Output file to write result to. Defaults to stdout.
    #[structopt(short, long, parse(from_os_str))]
    pub output: Option<PathBuf>,

    /// Output format. Either "tgf" graph format or "points" point cloud.
    #[structopt(short, long, default_value = "tgf")]
    pub format: OutputFormat,

    /// Spacing between joined together particles.
    #[structopt(long, default_value = "1")]
    pub particle_spacing: f64,

    /// Distance threshold for joining together two particles.
    #[structopt(short, long, default_value = "3")]
    pub attraction_distance: f64,

    /// Minimum move distance for random walk.
    #[structopt(short, long, default_value = "1")]
    pub min_move_distance: f64,

    /// Defines how many interactions are necessary for a particle to stick to another.
    /// The number of join attempts is tracked per-particle.
    #[structopt(long, default_value = "0")]
    pub stubbornness: usize,

    /// Defines the probability that another particle will allow a particle to stick to another.
    /// Applies after stubbornness.
    #[structopt(long, default_value = "1")]
    pub stickiness: f64,

    /// Number of seed particles.
    /// If one seed particle is used, it will be placed at the origin.
    /// Otherwise, the seed particles will be uniformly spread around the origin.
    #[structopt(long, default_value = "1")]
    pub seeds: usize,

    /// The random seed to use, for reproducibility. Zero for a random seed.
    #[structopt(long, default_value = "0")]
    pub seed: u64,

    // TODO: Need to define different methods of placing the seed points.

    /// Dimensionality of the particles.
    #[structopt(short, long, default_value = "2")]
    pub dimensions: u8,

    /// Number of particles to add.
    #[structopt(short, long, default_value = "10000")]
    pub particles: usize,
}

impl Options {
    /// Parse options from commandline arguments.
    pub fn from_args() -> Options {
        <Options as StructOpt>::from_args()
    }

    /// Get a BufWriter for stdout or the specified output file.
    pub fn get_output_writer(&self) -> BufWriter<Box<dyn Write>> {
        match &self.output {
            Some(path) => match File::create(&path) {
                Err(why) => panic!("Couldn't create: {} because: {}", path.display(), why),
                Ok(file) => {
                    trace!("Using file output: {}", path.display());
                    BufWriter::new(Box::new(file))
                }
            },
            None => {
                trace!("Using stdout output");
                BufWriter::new(Box::new(std::io::stdout()))
            }
        }
    }
}
