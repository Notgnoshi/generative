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

    // TODO: Allow a normal random number of points.
    /// The number of points to generate.
    #[clap(short, long)]
    pub num_points: usize,

    /// The random domain to generate points inside.
    #[clap(short, long, default_value = "unit-circle", value_enum)]
    pub domain: RandomDomain,

    /// Optionally scale the generated points.
    #[clap(short, long, default_value = "1.0")]
    pub scale: f64,
}
