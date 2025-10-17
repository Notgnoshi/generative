use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use generative::io::{get_input_reader, get_output_writer, read_geometries, write_geometries};
use geo::{
    AffineOps, AffineTransform, BoundingRect, Coord, Geometry, MapCoordsInPlace, Rect, coord,
};
use wkt::ToWkt;

#[derive(Debug, Clone, ValueEnum)]
enum TransformCenter {
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
struct CmdlineOptions {
    /// The log level
    #[clap(short, long, default_value_t = tracing::Level::INFO)]
    log_level: tracing::Level,

    /// Output file to write result to. Defaults to stdout.
    #[clap(short, long)]
    output: Option<PathBuf>,

    /// Input file to read input from. Defaults to stdin.
    #[clap(short, long)]
    input: Option<PathBuf>,

    /// How to center the affine transformation
    #[clap(long, default_value = "origin")]
    center: TransformCenter,

    /// Degrees CCW rotation, applied before any other transformation
    #[clap(short, long, default_value_t = 0.0)]
    rotation: f64,

    /// Symmetric multiplicative scale, applied after rotation, applied before any x or y scales
    #[clap(short, long)]
    scale: Option<f64>,

    /// The x multiplicative scale, applied after rotation
    #[clap(long)]
    scale_x: Option<f64>,

    /// The y multiplicative scale, applied after rotation
    #[clap(long)]
    scale_y: Option<f64>,

    /// The x additive offset, applied after scale
    #[clap(long)]
    offset_x: Option<f64>,

    /// The y additive offset, applied after scale
    #[clap(long)]
    offset_y: Option<f64>,

    /// Degrees x skew, applied after offset
    #[clap(long)]
    skew_x: Option<f64>,

    /// Degrees y skew, applied after offset
    #[clap(long)]
    skew_y: Option<f64>,

    /// Convert the input geometries from (x, y) to (r, theta)
    ///
    /// Any affine transformations are applied in the original coordinate space
    #[clap(long, conflicts_with = "from_polar")]
    to_polar: bool,

    /// Convert the input geometries from (r, theta) to (x, y)
    ///
    /// Any affine transformations are applied in the original coordinate space
    #[clap(long, conflicts_with = "to_polar")]
    from_polar: bool,

    /// Scale coordinate 1 (x, or r) to fit in the given range
    ///
    /// If specified, will be applied regardless of whether polar conversion is performed
    #[clap(long, num_args = 2)]
    range1: Vec<f64>,

    /// Scale coordinate 2 (y, or theta) to fit in the given range
    ///
    /// If specified, will be applied regardless of whether polar conversion is performed
    #[clap(long, num_args = 2)]
    range2: Vec<f64>,
}

fn build_transform(args: &CmdlineOptions, center: Coord) -> AffineTransform {
    let mut transform = AffineTransform::rotate(args.rotation, center);

    if let Some(scale) = args.scale {
        transform = transform.scaled(scale, scale, center);
    }
    match (args.scale_x, args.scale_y) {
        (Some(x), Some(y)) => {
            transform = transform.scaled(x, y, center);
        }
        (Some(x), None) => {
            transform = transform.scaled(x, 1.0, center);
        }
        (None, Some(y)) => {
            transform = transform.scaled(1.0, y, center);
        }
        (None, None) => {}
    }
    match (args.offset_x, args.offset_y) {
        (Some(x), Some(y)) => {
            transform = transform.translated(x, y);
        }
        (Some(x), None) => {
            transform = transform.translated(x, 0.0);
        }
        (None, Some(y)) => {
            transform = transform.translated(0.0, y);
        }
        (None, None) => {}
    }
    match (args.skew_x, args.skew_y) {
        (Some(x), Some(y)) => {
            transform = transform.skewed(x, y, center);
        }
        (Some(x), None) => {
            transform = transform.skewed(x, 0.0, center);
        }
        (None, Some(y)) => {
            transform = transform.skewed(0.0, y, center);
        }
        (None, None) => {}
    }

    transform
}

fn bounding_box(geometries: &[Geometry]) -> Rect {
    // Calculate the center of the bounding box; needed to build the AffineTransform
    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;
    for geom in geometries.iter() {
        let temp = geom.bounding_rect().unwrap_or_else(|| {
            panic!(
                "Geometry '{}' didn't have a bounding rectangle",
                geom.to_wkt()
            )
        });

        let min = temp.min();
        let max = temp.max();

        min_x = min_x.min(min.x);
        min_y = min_y.min(min.y);
        max_x = max_x.max(max.x);
        max_y = max_y.max(max.y);
    }
    Rect::new(coord! {x:min_x, y:min_y}, coord! {x:max_x, y:max_y})
}

fn affine_transform(
    geometries: impl Iterator<Item = Geometry> + 'static,
    args: &CmdlineOptions,
) -> Box<dyn Iterator<Item = Geometry> + '_> {
    match args.center {
        TransformCenter::Origin => {
            let center = coord! {x:0.0, y: 0.0};
            let transform = build_transform(args, center);
            Box::new(geometries.map(move |geom| geom.affine_transform(&transform)))
        }
        TransformCenter::EachGeometry => {
            let map = geometries.map(move |geom| {
                let center = geom
                    .bounding_rect()
                    .unwrap_or_else(|| {
                        panic!(
                            "Geometry '{}' didn't have a bounding rectangle",
                            geom.to_wkt()
                        )
                    })
                    .center();
                let transform = build_transform(args, center);
                geom.affine_transform(&transform)
            });
            Box::new(map)
        }
        // more expensive for large numbers of geometries (has to load all of them into RAM before
        // performing the transformations)
        TransformCenter::WholeCollection => {
            let geometries: Vec<_> = geometries.collect();
            let rect = bounding_box(&geometries);
            let center = rect.center();
            let transform = build_transform(args, center);

            // Instead of applying the transformation in-place all at once _and then_ writing the
            // results, we lazily perform the transformation so that we can pipeline the
            // transformation and the serialization.
            let map = geometries
                .into_iter()
                .map(move |geom| geom.affine_transform(&transform));
            Box::new(map)
        }
    }
}

fn from_polar(coord: Coord) -> Coord {
    let r = coord.x;
    let theta = coord.y;
    coord! { x: r * f64::cos(theta), y: r * f64::sin(theta) }
}

fn to_polar(coord: Coord) -> Coord {
    let r = f64::sqrt(coord.x.powi(2) + coord.y.powi(2));
    let mut theta = f64::atan2(coord.y, coord.x);
    if theta < 0.0 {
        theta += 2.0 * std::f64::consts::PI;
    }
    coord! { x: r, y: theta}
}

fn scale_range(src: &[f64; 2], dst: &[f64; 2], v: f64) -> f64 {
    (dst[1] - dst[0]) * (v - src[0]) / (src[1] - src[0]) + dst[0]
}

fn scale_coord_range(
    bounds: &Rect,
    x_dst: Option<&[f64; 2]>,
    y_dst: Option<&[f64; 2]>,
    coord: Coord,
) -> Coord {
    let min = bounds.min();
    let max = bounds.max();
    let mut x = coord.x;
    let mut y = coord.y;

    if let Some(dst) = x_dst {
        let src = [min.x, max.x];
        x = scale_range(&src, dst, coord.x);
    }
    if let Some(dst) = y_dst {
        let src = [min.y, max.y];
        y = scale_range(&src, dst, coord.y);
    }
    coord! {x: x, y: y}
}

fn geoms_coordwise(
    geometries: impl Iterator<Item = Geometry>,
    transform: impl Fn(Coord) -> Coord + Copy,
) -> impl Iterator<Item = Geometry> {
    geometries.into_iter().map(move |mut geom| {
        geom.map_coords_in_place(transform);
        geom
    })
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

    let reader = get_input_reader(&args.input)?;
    let writer = get_output_writer(&args.output)?;
    let geometries = read_geometries(reader);
    let mut transformed = affine_transform(geometries, &args);

    if args.range1.len() == 2 || args.range2.len() == 2 {
        let geometries: Vec<_> = transformed.collect();
        let bounds = bounding_box(&geometries);

        let mut x_dst = None;
        let mut y_dst = None;
        if args.range1.len() == 2 {
            let dst = [args.range1[0], args.range1[1]];
            x_dst = Some(dst);
        }
        if args.range2.len() == 2 {
            // If we're converting from polar, then the "y" coordinate is actually theta.
            // Use degrees in the CLI args, because it's waaaay easier to do "0 360" than it is
            // "0 2PI"
            let dst = if args.from_polar {
                [args.range2[0].to_radians(), args.range2[1].to_radians()]
            } else {
                [args.range2[0], args.range2[1]]
            };
            y_dst = Some(dst);
        }

        let scaled: Vec<_> = geoms_coordwise(geometries.into_iter(), |coord| {
            scale_coord_range(&bounds, x_dst.as_ref(), y_dst.as_ref(), coord)
        })
        .collect();
        transformed = Box::new(scaled.into_iter());
    }

    if args.to_polar {
        transformed = Box::new(geoms_coordwise(transformed, to_polar));
    } else if args.from_polar {
        transformed = Box::new(geoms_coordwise(transformed, from_polar));
    }

    write_geometries(writer, transformed)
}
