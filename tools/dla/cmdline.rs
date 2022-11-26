use clap::{Parser, ValueEnum};
use log::trace;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

/// Specifies the plaintext output format.
/// In all cases, the seed points will be written first.
#[derive(Debug, Clone, ValueEnum)]
pub enum OutputFormat {
    /// Use TGF graph output format, labelling each node with the WKT point.
    /// Edges will be unlabeled with weight, as that can be done in a post-processing
    /// step, which simplifies the datastructure here.
    Tgf,
    /// Prints each point in WKT on its own line.
    Wkt,
}

/// Off-lattice diffusion limited aggregation
#[derive(Debug, Parser)]
#[clap(name = "dla")]
pub struct CmdlineOptions {
    /// Silence all logging
    #[clap(short, long)]
    pub quiet: bool,

    /// Increase logging verbosity.
    #[clap(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Output file to write result to. Defaults to stdout.
    #[clap(short, long)]
    pub output: Option<PathBuf>,

    /// Output format. Either "tgf" graph format or "points" point cloud.
    #[clap(short, long, default_value = "tgf")]
    pub format: OutputFormat,

    /// Spacing between joined together particles.
    #[clap(long, default_value = "1")]
    pub particle_spacing: f64,

    /// Distance threshold for joining together two particles.
    #[clap(short, long, default_value = "3")]
    pub attraction_distance: f64,

    /// Minimum move distance for random walk.
    #[clap(short, long, default_value = "1")]
    pub min_move_distance: f64,

    /// Defines how many interactions are necessary for a particle to stick to another.
    /// The number of join attempts is tracked per-particle.
    #[clap(long, default_value = "0")]
    pub stubbornness: usize,

    /// Defines the probability that another particle will allow a particle to stick to another.
    /// Applies after stubbornness.
    #[clap(long, default_value = "1")]
    pub stickiness: f64,

    /// Number of seed particles.
    /// If one seed particle is used, it will be placed at the origin.
    /// Otherwise, the seed particles will be uniformly spread around the origin.
    #[clap(long, default_value = "1")]
    pub seeds: usize,

    /// The random seed to use, for reproducibility. Zero for a random seed.
    #[clap(long, default_value = "0")]
    pub seed: u64,

    // TODO: Need to define different methods of placing the seed points.
    /// Dimensionality of the particles.
    #[clap(short, long, default_value = "2")]
    pub dimensions: u8,

    /// Number of particles to add.
    #[clap(short, long, default_value = "10000")]
    pub particles: usize,
}

impl CmdlineOptions {
    /// Get a BufWriter for stdout or the specified output file.
    pub fn get_output_writer(&self) -> BufWriter<Box<dyn Write>> {
        match &self.output {
            Some(path) => match File::create(path) {
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
