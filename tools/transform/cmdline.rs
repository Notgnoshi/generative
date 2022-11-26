use clap::{Parser, ValueEnum};
use std::path::PathBuf;

use generative::wkio::GeometryFormat;

#[derive(Debug, Clone, ValueEnum)]
pub enum TransformCenter {
    /// Center the affine transform on (0, 0)
    Origin,
    /// Center the transform on the center of each geometry's bounding box
    EachGeometry,
    /// Center the transform on the center of the entire collection's bounding box
    WholeCollection,
}

/// Perform transformations on 2D geometries
///
/// Transformations are applied in the order:
///
/// 1. rotation
/// 2. scale
/// 3. offset
/// 4. skew
///
/// If you want to apply transformations in any other order, you can chain invocations of this
/// command, specifying only one transformation per invocation.
///
/// If you want to apply transformations to 3D geometries, they must first be projected to 2D using
/// the project.py tool.
#[derive(Debug, Parser)]
#[clap(name = "transform", verbatim_doc_comment)]
pub struct CmdlineOptions {
    /// Increase logging verbosity. Defaults to ERROR level.
    #[clap(short, long, action = clap::ArgAction::Count)]
    pub verbosity: u8,

    /// Output file to write result to. Defaults to stdout.
    #[clap(short, long)]
    pub output: Option<PathBuf>,

    /// Output geometry format.
    #[clap(short = 'O', long, default_value_t = GeometryFormat::Wkt)]
    pub output_format: GeometryFormat,

    /// Input file to read input from. Defaults to stdin.
    #[clap(short, long)]
    pub input: Option<PathBuf>,

    /// Input geometry format.
    #[clap(short = 'I', long, default_value_t = GeometryFormat::Wkt)]
    pub input_format: GeometryFormat,

    /// How to center the affine transformation
    #[clap(long, default_value = "origin")]
    pub center: TransformCenter,

    /// Degrees CCW rotation, applied before any other transformation
    #[clap(short, long, default_value_t = 0.0)]
    pub rotation: f64,

    /// The (x, y) multiplicative scale, applied after rotation
    #[clap(short = 's', long, number_of_values = 2)]
    pub scale: Option<Vec<f64>>,

    /// The (x, y) additive offset, applied after scale
    #[clap(short = 't', long, number_of_values = 2)]
    pub offset: Option<Vec<f64>>,

    /// Degrees (x, y) skew, applied after offset
    #[clap(short = 'S', long, number_of_values = 2)]
    pub skew: Option<Vec<f64>>,
}
