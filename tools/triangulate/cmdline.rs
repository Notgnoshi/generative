use clap::{Parser, ValueEnum};
use generative::wkio::GeometryFormat;
use std::path::PathBuf;

#[derive(Debug, Clone, ValueEnum)]
pub enum TriangulationStrategy {
    /// Triangulate each geometry individually
    ///
    /// Meaningful for polygons, not so much for points and linestrings.
    EachGeometry,
    /// Collapse the whole geometry collection into a point cloud to triangulate
    WholeCollection,
}

/// Triangulate the given geometries
#[derive(Debug, Parser)]
#[clap(name = "triangulate", verbatim_doc_comment)]
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

    /// How to triangulate the input geometries
    #[clap(short, long, default_value = "whole-collection")]
    pub strategy: TriangulationStrategy,
}
