use std::io::Write;
use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use generative::io::{get_output_writer, write_geometries, GeometryFormat};
use geo::{Geometry, Line, Point};
use itertools::Itertools;
use stderrlog::ColorChoice;

/// Perform bitwise operations on a grid
///
/// <https://www.reddit.com/r/generative/comments/10hk4jg/big_renfest_crest_energy_bitwise_operations_svg>
#[derive(Debug, Parser)]
#[clap(name = "bitwise", verbatim_doc_comment)]
struct CmdlineOptions {
    /// The log level
    #[clap(short, long, default_value_t = log::Level::Info)]
    pub log_level: log::Level,

    /// Output file to write result to. Defaults to stdout.
    #[clap(short, long)]
    pub output: Option<PathBuf>,

    /// Output geometry format.
    #[clap(short = 'O', long, default_value_t = GeometryFormat::Wkt)]
    pub output_format: GeometryFormat,

    /// Whether to output points or connected lines
    #[clap(long, default_value_t = false)]
    pub lines: bool,

    /// Maximum x coordinate
    #[clap(short = 'x', long, default_value_t = 0)]
    pub x_min: usize,

    /// Maximum x coordinate
    #[clap(short = 'X', long, default_value_t = 48)]
    pub x_max: usize,

    /// Minimum y coordinate
    #[clap(short = 'y', long, default_value_t = 0)]
    pub y_min: usize,

    /// Maximum y coordinate
    #[clap(short = 'Y', long, default_value_t = 48)]
    pub y_max: usize,

    /// The order to search adjacent cells for neighbors. Comma separated.
    #[clap(short, long,
           use_value_delimiter = true,
           value_delimiter = ',',
           default_values_t = [Neighbor::East, Neighbor::SouthEast, Neighbor::South, Neighbor::SouthWest]
    )]
    pub neighbor_search_order: Vec<Neighbor>,
}

fn expression(x: usize, y: usize) -> usize {
    // power of two grid structure because of ANDing x & y
    (x & y) & ((x ^ y) % 13)
}

fn write_line<W>(writer: W, format: &GeometryFormat, x1: usize, y1: usize, x2: usize, y2: usize)
where
    W: Write,
{
    let line = Line::new(
        Point::new(x1 as f64, y1 as f64),
        Point::new(x2 as f64, y2 as f64),
    );
    let geometries = std::iter::once(Geometry::Line(line));
    write_geometries(writer, geometries, format);
}

fn write_point<W>(writer: W, format: &GeometryFormat, x1: usize, y1: usize)
where
    W: Write,
{
    let point = Point::new(x1 as f64, y1 as f64);
    let geometries = std::iter::once(Geometry::Point(point));
    write_geometries(writer, geometries, format);
}

#[derive(Debug, Clone, PartialEq, Eq, ValueEnum)]
enum Neighbor {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}

impl std::fmt::Display for Neighbor {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            // important: Should match clap::ValueEnum format
            Neighbor::North => write!(f, "north"),
            Neighbor::NorthEast => write!(f, "north-east"),
            Neighbor::East => write!(f, "east"),
            Neighbor::SouthEast => write!(f, "south-east"),
            Neighbor::South => write!(f, "south"),
            Neighbor::SouthWest => write!(f, "south-west"),
            Neighbor::West => write!(f, "west"),
            Neighbor::NorthWest => write!(f, "north-west"),
        }
    }
}

fn neighbor(x: usize, y: usize, n: Neighbor) -> (usize, usize) {
    match n {
        Neighbor::North => (x, y - 1),
        Neighbor::NorthEast => (x + 1, y - 1),
        Neighbor::East => (x + 1, y),
        Neighbor::SouthEast => (x + 1, y + 1),
        Neighbor::South => (x, y + 1),
        Neighbor::SouthWest => (x - 1, y + 1),
        Neighbor::West => (x - 1, y),
        Neighbor::NorthWest => (x - 1, y - 1),
    }
}

fn main() {
    let args = CmdlineOptions::parse();

    stderrlog::new()
        .verbosity(args.log_level)
        .color(ColorChoice::Auto)
        .init()
        .expect("Failed to initialize stderrlog");

    let xs = args.x_min..args.x_max;
    let ys = args.y_min..args.y_max;
    let cross = xs.cartesian_product(ys);

    let mut writer = get_output_writer(&args.output).unwrap();
    if args.lines {
        log::info!(
            "Searching neighbors in order: {:?}",
            args.neighbor_search_order
        );
        for (x, y) in cross {
            if expression(x, y) > 0 {
                let mut wrote_line = false;
                for n in args.neighbor_search_order.iter() {
                    let (x2, y2) = neighbor(x, y, n.clone());
                    if expression(x2, y2) > 0 {
                        write_line(&mut writer, &args.output_format, x, y, x2, y2);
                        wrote_line = true;
                        break;
                    }
                }
                if !wrote_line {
                    write_point(&mut writer, &args.output_format, x, y);
                }
            }
        }
    } else {
        let geometries = cross.filter_map(|(x, y)| {
            if expression(x, y) > 0 {
                Some(Geometry::Point(Point::new(x as f64, y as f64)))
            } else {
                None
            }
        });

        write_geometries(writer, geometries, &args.output_format);
    }
}
