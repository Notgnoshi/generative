mod cmdline;

use clap::Parser;
use generative::flatten::flatten_geometries_into_points;
use generative::stdio::{get_input_reader, get_output_writer};
use generative::triangulation::{triangulate, write_graph};
use generative::wkio::read_geometries;
use stderrlog::ColorChoice;

fn main() {
    let args = cmdline::CmdlineOptions::parse();

    stderrlog::new()
        .verbosity(args.verbosity as usize + 1) // Default to WARN level.
        .color(ColorChoice::Auto)
        .init()
        .expect("Failed to initialize stderrlog");

    let reader = get_input_reader(&args.input).unwrap();
    let geometries = read_geometries(reader, &args.input_format); // lazily loaded

    let points = flatten_geometries_into_points(geometries);
    let triangulation = triangulate(points);
    let urquhart = triangulation.urquhart();

    let writer = get_output_writer(&args.output).unwrap();
    write_graph(writer, urquhart, &args.output_format);
}
