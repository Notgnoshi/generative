use std::io::Write;
use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use generative::io::{get_output_writer, write_geometries, GeometryFormat};
use geo::{Geometry, Line, Point};
use itertools::Itertools;
use rhai::{Engine, EvalAltResult, Scope, AST};
use stderrlog::ColorChoice;

/// Perform bitwise operations on a grid
///
/// <https://www.reddit.com/r/generative/comments/10hk4jg/big_renfest_crest_energy_bitwise_operations_svg>
#[derive(Debug, Parser)]
#[clap(name = "bitwise", verbatim_doc_comment)]
struct CmdlineOptions {
    /// The log level
    #[clap(short, long, default_value_t = log::Level::Info)]
    log_level: log::Level,

    /// Output file to write result to. Defaults to stdout.
    #[clap(short, long)]
    output: Option<PathBuf>,

    /// Output geometry format.
    #[clap(short = 'O', long, default_value_t = GeometryFormat::Wkt)]
    output_format: GeometryFormat,

    /// Whether to output points or connected lines
    #[clap(long, default_value_t = false)]
    points: bool,

    /// Maximum x coordinate
    #[clap(short = 'x', long, default_value_t = 0)]
    x_min: i64,

    /// Maximum x coordinate
    #[clap(short = 'X', long, default_value_t = 48)]
    x_max: i64,

    /// Minimum y coordinate
    #[clap(short = 'y', long, default_value_t = 0)]
    y_min: i64,

    /// Maximum y coordinate
    #[clap(short = 'Y', long, default_value_t = 48)]
    y_max: i64,

    /// The order to search adjacent cells for neighbors. Comma separated.
    #[clap(short, long,
           use_value_delimiter = true,
           value_delimiter = ',',
           default_values_t = [Neighbor::East, Neighbor::SouthEast, Neighbor::South, Neighbor::SouthWest]
    )]
    neighbor_search_order: Vec<Neighbor>,

    /// A valid Rust expression taking 'x' and 'y', and returning an i64
    #[clap(default_value = "(x & y) & (x ^ y) % 13")]
    expression: String,
}

fn expression(engine: &Engine, ast: &AST, x: i64, y: i64) -> Result<i64, Box<EvalAltResult>> {
    let mut scope = Scope::new();
    scope.push("x", x);
    scope.push("y", y);

    engine.eval_ast_with_scope::<i64>(&mut scope, ast)
}

fn write_line<W>(writer: W, format: &GeometryFormat, x1: i64, y1: i64, x2: i64, y2: i64)
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

fn write_point<W>(writer: W, format: &GeometryFormat, x1: i64, y1: i64)
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

fn neighbor(x: i64, y: i64, n: Neighbor) -> (i64, i64) {
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

fn main() -> Result<(), Box<EvalAltResult>> {
    let args = CmdlineOptions::parse();

    stderrlog::new()
        .verbosity(args.log_level)
        .color(ColorChoice::Auto)
        .init()
        .expect("Failed to initialize stderrlog");

    let engine = Engine::new();
    let ast = engine.compile_expression(&args.expression)?;

    let xs = args.x_min..args.x_max;
    let ys = args.y_min..args.y_max;
    let cross = xs.cartesian_product(ys);

    let mut writer = get_output_writer(&args.output).unwrap();
    if args.points {
        let geometries = cross.filter_map(|(x, y)| {
            if let Ok(value) = expression(&engine, &ast, x, y) {
                if value > 0 {
                    return Some(Geometry::Point(Point::new(x as f64, y as f64)));
                }
            } else {
                log::error!(
                    "Failed to evaluate expression '{}' given x={}, y={}",
                    args.expression,
                    x,
                    y
                );
            }
            None
        });

        write_geometries(writer, geometries, &args.output_format);
    } else {
        log::info!(
            "Searching neighbors in order: {:?}",
            args.neighbor_search_order
        );
        for (x, y) in cross {
            if expression(&engine, &ast, x, y)? > 0 {
                let mut wrote_line = false;
                for n in args.neighbor_search_order.iter() {
                    let (x2, y2) = neighbor(x, y, n.clone());
                    if expression(&engine, &ast, x2, y2)? > 0 {
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
    }

    Ok(())
}
