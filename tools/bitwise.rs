use std::io::Write;
use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use generative::io::{get_output_writer, write_geometries};
use geo::{Geometry, Line, Point};
use itertools::Itertools;

/// Perform bitwise operations on a grid
///
/// <https://www.reddit.com/r/generative/comments/10hk4jg/big_renfest_crest_energy_bitwise_operations_svg>
#[derive(Debug, Parser)]
#[clap(name = "bitwise", verbatim_doc_comment)]
struct CmdlineOptions {
    /// The log level
    #[clap(short, long, default_value_t = tracing::Level::INFO)]
    log_level: tracing::Level,

    /// Output file to write result to. Defaults to stdout.
    #[clap(short, long)]
    output: Option<PathBuf>,

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

type BitwiseExpression = Box<dyn Fn(i64, i64) -> i64>;

fn build_expression(expr: &str) -> eyre::Result<BitwiseExpression> {
    // Ok(Box::new(|x, y| (x & y) & ((x ^ y) % 13)))

    let context = rune::Context::with_default_modules()?;
    let runtime = rune::sync::Arc::try_new(context.runtime()?)?;
    let mut script = rune::Sources::new();
    script.insert(build_function(expr)?)?;
    let mut diagnostics = rune::Diagnostics::new();
    let maybe_unit = rune::prepare(&mut script)
        .with_context(&context)
        .with_diagnostics(&mut diagnostics)
        .build();
    if !diagnostics.is_empty() {
        let mut writer =
            rune::termcolor::StandardStream::stderr(rune::termcolor::ColorChoice::Always);
        diagnostics.emit(&mut writer, &script)?;
    }
    let unit = rune::sync::Arc::try_new(maybe_unit?)?;
    let vm = rune::Vm::new(runtime, unit);
    let eval_expr = vm.lookup_function(["eval_expr"])?;
    let eval_expr = move |x: i64, y: i64| -> i64 {
        eval_expr.call((x, y)).expect("Failed to call eval_expr()")
    };
    Ok(Box::new(eval_expr))
}

fn build_function(expr: &str) -> eyre::Result<rune::Source> {
    let lines = ["pub fn eval_expr(x, y) {", expr, "}"];
    let script = lines.join("\n");
    let source = rune::Source::memory(script)?;
    Ok(source)
}

fn write_line<W>(writer: W, x1: i64, y1: i64, x2: i64, y2: i64) -> eyre::Result<()>
where
    W: Write,
{
    let line = Line::new(
        Point::new(x1 as f64, y1 as f64),
        Point::new(x2 as f64, y2 as f64),
    );
    let geometries = std::iter::once(Geometry::Line(line));
    write_geometries(writer, geometries)
}

fn write_point<W>(writer: W, x1: i64, y1: i64) -> eyre::Result<()>
where
    W: Write,
{
    let point = Point::new(x1 as f64, y1 as f64);
    let geometries = std::iter::once(Geometry::Point(point));
    write_geometries(writer, geometries)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
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

fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    let args = CmdlineOptions::parse();
    let filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(args.log_level.into())
        .from_env_lossy();
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_ansi(true)
        .with_writer(std::io::stderr)
        .init();

    let expr_fn = build_expression(&args.expression)?;

    let xs = args.x_min..args.x_max;
    let ys = args.y_min..args.y_max;
    let cross = xs.cartesian_product(ys);

    let mut writer = get_output_writer(&args.output)?;
    if args.points {
        let geometries = cross.filter_map(|(x, y)| {
            let value = expr_fn(x, y);
            if value > 0 {
                return Some(Geometry::Point(Point::new(x as f64, y as f64)));
            }
            None
        });

        write_geometries(writer, geometries)?;
    } else {
        tracing::info!(
            "Searching neighbors in order: {:?}",
            args.neighbor_search_order
        );
        for (x, y) in cross {
            if expr_fn(x, y) > 0 {
                let mut wrote_line = false;
                for n in args.neighbor_search_order.iter() {
                    let (x2, y2) = neighbor(x, y, *n);
                    if expr_fn(x2, y2) > 0 {
                        write_line(&mut writer, x, y, x2, y2)?;
                        wrote_line = true;
                        break;
                    }
                }
                if !wrote_line {
                    write_point(&mut writer, x, y)?;
                }
            }
        }
    }

    Ok(())
}
