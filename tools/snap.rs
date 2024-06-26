use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use generative::graph::GeometryGraph;
use generative::io::{
    get_input_reader, get_output_writer, read_geometries, read_tgf_graph, write_geometries,
    write_tgf_graph, GeometryFormat,
};
use generative::snap::{snap_geoms, snap_graph, SnappingStrategy};
use petgraph::Undirected;
use stderrlog::ColorChoice;

#[derive(Debug, Clone, ValueEnum)]
enum InputFormat {
    /// One WKT geometry per line. Ignores trailing garbage; does not skip over leading garbage.
    Wkt,
    /// Stringified hex encoded WKB, one geometry per line
    WkbHex,
    /// Raw WKB bytes with no separator between geometries
    WkbRaw,
    /// A geometry graph in TGF format
    Tgf,
}

impl std::fmt::Display for InputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            // important: Should match clap::ValueEnum format
            InputFormat::Wkt => write!(f, "wkt"),
            InputFormat::WkbHex => write!(f, "wkb-hex"),
            InputFormat::WkbRaw => write!(f, "wkb-raw"),
            InputFormat::Tgf => write!(f, "tgf"),
        }
    }
}

impl From<InputFormat> for GeometryFormat {
    fn from(f: InputFormat) -> GeometryFormat {
        match f {
            InputFormat::Wkt => GeometryFormat::Wkt,
            InputFormat::WkbHex => GeometryFormat::WkbHex,
            InputFormat::WkbRaw => GeometryFormat::WkbRaw,
            InputFormat::Tgf => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, ValueEnum)]
enum CliSnappingStrategy {
    ClosestPoint,
    RegularGrid,
}

impl std::fmt::Display for CliSnappingStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            // important: Should match clap::ValueEnum format
            CliSnappingStrategy::ClosestPoint => write!(f, "closest-point"),
            CliSnappingStrategy::RegularGrid => write!(f, "regular-grid"),
        }
    }
}

/// Snap close-together vertices on geometries
#[derive(Debug, Parser)]
#[clap(name = "snap", verbatim_doc_comment)]
struct CmdlineOptions {
    /// The log level
    #[clap(short, long, default_value_t = log::Level::Info)]
    log_level: log::Level,

    /// Input file to read input from. Defaults to stdin.
    #[clap(short, long)]
    input: Option<PathBuf>,

    /// Input geometry format.
    #[clap(short = 'I', long, default_value_t = InputFormat::Wkt)]
    input_format: InputFormat,

    /// Output file to write result to. Defaults to stdout.
    #[clap(short, long)]
    output: Option<PathBuf>,

    /// The strategy to use for snapping
    #[clap(short, long, default_value_t = CliSnappingStrategy::ClosestPoint)]
    strategy: CliSnappingStrategy,

    /// The tolerance to use when snapping
    #[clap(short, long, default_value_t = 0.001)]
    tolerance: f64,
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
    let strategy = match args.strategy {
        CliSnappingStrategy::ClosestPoint => SnappingStrategy::ClosestPoint(args.tolerance),
        CliSnappingStrategy::RegularGrid => SnappingStrategy::RegularGrid(args.tolerance),
    };

    match args.input_format {
        InputFormat::Wkt | InputFormat::WkbHex | InputFormat::WkbRaw => {
            let geometries = read_geometries(reader, &args.input_format.clone().into());
            let geometries = snap_geoms(geometries, strategy);
            write_geometries(writer, geometries, args.input_format.into());
        }
        InputFormat::Tgf => {
            let graph: GeometryGraph<Undirected> = read_tgf_graph(reader);
            let graph = snap_graph(graph, strategy);
            write_tgf_graph(&mut writer, &graph);
        }
    }
}
