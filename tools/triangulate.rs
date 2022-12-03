use clap::{Parser, ValueEnum};
use generative::flatten::flatten_geometries_into_points;
use generative::stdio::{get_input_reader, get_output_writer};
use generative::triangulation::triangulate;
use generative::wkio::{read_geometries, write_geometries, GeometryFormat};
use std::path::PathBuf;
use stderrlog::ColorChoice;

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

fn main() {
    let args = CmdlineOptions::parse();

    stderrlog::new()
        .verbosity(args.verbosity as usize + 1) // Default to WARN level.
        .color(ColorChoice::Auto)
        .init()
        .expect("Failed to initialize stderrlog");

    let reader = get_input_reader(&args.input).unwrap();
    let mut writer = get_output_writer(&args.output).unwrap();
    let geometries = read_geometries(reader, &args.input_format); // lazily loaded

    match args.strategy {
        TriangulationStrategy::EachGeometry => {
            let triangulations = geometries
                .map(|geom| flatten_geometries_into_points(std::iter::once(geom)))
                .map(triangulate);
            for triangulation in triangulations {
                let lines = triangulation.lines().map(geo::Geometry::Line);
                write_geometries(&mut writer, lines, &args.output_format);
            }
        }
        TriangulationStrategy::WholeCollection => {
            let points = flatten_geometries_into_points(geometries);
            let triangulation = triangulate(points);
            let lines = triangulation.lines().map(geo::Geometry::Line);
            write_geometries(writer, lines, &args.output_format);
        }
    }
}
