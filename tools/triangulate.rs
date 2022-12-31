use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use generative::flatten::flatten_geometries_into_points;
use generative::io::{
    get_input_reader, get_output_writer, read_geometries, write_graph, GeometryFormat, GraphFormat,
};
use generative::triangulation::triangulate;
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
    /// The log level
    #[clap(short, long, default_value_t = log::Level::Info)]
    pub log_level: log::Level,

    /// Output file to write result to. Defaults to stdout.
    #[clap(short, long)]
    pub output: Option<PathBuf>,

    /// Output geometry format.
    #[clap(short = 'O', long, default_value_t = GraphFormat::Wkt)]
    pub output_format: GraphFormat,

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
        .verbosity(args.log_level)
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
                let graph = triangulation.graph();
                write_graph(&mut writer, graph, &args.output_format);
            }
        }
        TriangulationStrategy::WholeCollection => {
            let points = flatten_geometries_into_points(geometries);
            let triangulation = triangulate(points);
            let graph = triangulation.graph();
            write_graph(writer, graph, &args.output_format);
        }
    }
}
