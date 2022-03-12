use clap::Parser;
use std::path::PathBuf;

#[derive(Debug)]
pub enum RandomDomain {
    UnitSquare,
    UnitCircle,
}

impl std::str::FromStr for RandomDomain {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();
        let s = s.as_str();
        match s {
            "square" => Ok(RandomDomain::UnitSquare),
            "circle" => Ok(RandomDomain::UnitCircle),
            _ => Err(format!("Could not convert '{}' to RandomDomain", s)),
        }
    }
}

/// Generate random point clouds in a unit square or circle
#[derive(Debug, Parser)]
#[clap(name = "point-cloud")]
pub struct CmdlineOptions {
    /// Output file to write result to. Defaults to stdout.
    #[clap(short, long, parse(from_os_str))]
    pub output: Option<PathBuf>,

    // TODO: Allow a normal random number of points.
    /// The number of points to generate.
    #[clap(short, long)]
    pub num_points: usize,

    /// The random domain to generate points inside.
    /// One of "circle" or "square".
    #[clap(short, long, default_value = "circle")]
    pub domain: RandomDomain,

    /// Optionally scale the generated points.
    #[clap(short, long, default_value = "1.0")]
    pub scale: f64,
}
