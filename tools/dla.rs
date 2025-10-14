use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use generative::dla::{Model, format_tgf, format_wkt};

/// Specifies the plaintext output format.
/// In all cases, the seed points will be written first.
#[derive(Debug, Clone, ValueEnum)]
enum OutputFormat {
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
struct CmdlineOptions {
    /// The log level
    #[clap(short, long, default_value_t = tracing::Level::INFO)]
    log_level: tracing::Level,

    /// Output file to write result to. Defaults to stdout.
    #[clap(short, long)]
    output: Option<PathBuf>,

    /// Output format. Either "tgf" graph format or "points" point cloud.
    #[clap(short, long, default_value = "tgf")]
    format: OutputFormat,

    /// Spacing between joined together particles.
    #[clap(long, default_value = "1")]
    particle_spacing: f64,

    /// Distance threshold for joining together two particles.
    #[clap(short, long, default_value = "3")]
    attraction_distance: f64,

    /// Minimum move distance for random walk.
    #[clap(short, long, default_value = "1")]
    min_move_distance: f64,

    /// Defines how many interactions are necessary for a particle to stick to another.
    /// The number of join attempts is tracked per-particle.
    #[clap(long, default_value = "0")]
    stubbornness: usize,

    /// Defines the probability that another particle will allow a particle to stick to another.
    /// Applies after stubbornness.
    #[clap(long, default_value = "1")]
    stickiness: f64,

    /// Number of seed particles.
    /// If one seed particle is used, it will be placed at the origin.
    /// Otherwise, the seed particles will be uniformly spread around the origin.
    #[clap(long, default_value = "1")]
    seeds: usize,

    /// The random seed to use, for reproducibility. Zero for a random seed.
    #[clap(long, default_value = "0")]
    seed: u64,

    // TODO: Need to define different methods of placing the seed points.
    /// Dimensionality of the particles.
    #[clap(short, long, default_value = "2")]
    dimensions: u8,

    /// Number of particles to add.
    #[clap(short, long, default_value = "10000")]
    particles: usize,
}

impl CmdlineOptions {
    /// Get a BufWriter for stdout or the specified output file.
    fn get_output_writer(&self) -> BufWriter<Box<dyn Write>> {
        match &self.output {
            Some(path) => match File::create(path) {
                Err(why) => panic!("Couldn't create: {} because: {}", path.display(), why),
                Ok(file) => {
                    tracing::trace!("Using file output: {}", path.display());
                    BufWriter::new(Box::new(file))
                }
            },
            None => {
                tracing::trace!("Using stdout output");
                BufWriter::new(Box::new(std::io::stdout()))
            }
        }
    }
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

    let mut model = Model::new(
        args.dimensions,
        // TODO: Seed type.
        args.seeds,
        args.seed,
        args.particle_spacing,
        args.attraction_distance,
        args.min_move_distance,
        args.stubbornness,
        args.stickiness,
    );

    model.run(args.particles);

    tracing::trace!("Model {model:?}");

    let mut writer = args.get_output_writer();
    match args.format {
        OutputFormat::Tgf => format_tgf(&mut writer, model.particle_graph),
        OutputFormat::Wkt => format_wkt(&mut writer, model.particle_graph),
    }
}
