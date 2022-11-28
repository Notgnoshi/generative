mod cmdline;
use cmdline::TriangulationStrategy;

use clap::Parser;
use generative::flatten::flatten_geometries_into_points;
use generative::stdio::{get_input_reader, get_output_writer};
use generative::triangulation::triangulate;
use generative::wkio::{read_geometries, write_geometries};
use stderrlog::ColorChoice;

fn main() {
    let args = cmdline::CmdlineOptions::parse();

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
