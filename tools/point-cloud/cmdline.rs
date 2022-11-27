use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Debug, Clone, ValueEnum)]
pub enum RandomDomain {
    UnitSquare,
    UnitCircle,
}

/// Generate random point clouds in a unit square or circle
#[derive(Debug, Parser)]
#[clap(name = "point-cloud")]
pub struct CmdlineOptions {
    /// Output file to write result to. Defaults to stdout.
    #[clap(short, long)]
    pub output: Option<PathBuf>,

    /// The random seed to use. Use zero to let the tool pick its own random seed.
    #[clap(long, default_value = "0")]
    pub seed: u64,

    /// The number of points to generate.
    #[clap(short, long)]
    pub points: u64,

    /// Generate a random number of points with mean '--points'
    #[clap(short, long)]
    pub random_number: bool,

    /// The random domain to generate points inside.
    #[clap(short, long, default_value = "unit-circle", value_enum)]
    pub domain: RandomDomain,

    /// Scale the generated points.
    #[clap(short, long, default_value = "1.0")]
    pub scale: f64,
}
