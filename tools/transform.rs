use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use generative::io::{
    get_input_reader, get_output_writer, read_geometries, write_geometries, GeometryFormat,
};
use geo::{coord, AffineOps, AffineTransform, BoundingRect, Coord, Geometry, Rect};
use stderrlog::ColorChoice;
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
    #[clap(short, long, default_value_t = log::Level::Info)]
    log_level: log::Level,

    /// Output file to write result to. Defaults to stdout.
    #[clap(short, long)]
    output: Option<PathBuf>,

    /// Output geometry format.
    #[clap(short = 'O', long, default_value_t = GeometryFormat::Wkt)]
    output_format: GeometryFormat,

    /// Input file to read input from. Defaults to stdin.
    #[clap(short, long)]
    input: Option<PathBuf>,

    /// Input geometry format.
    #[clap(short = 'I', long, default_value_t = GeometryFormat::Wkt)]
    input_format: GeometryFormat,

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
            // Read geometries into memory so we can loop over them twice
            let geometries: Vec<Geometry<f64>> = geometries.collect();
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
            let rect = Rect::new(coord! {x:min_x, y:min_y}, coord! {x:max_x, y:max_y});
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

fn main() {
    let args = CmdlineOptions::parse();

    stderrlog::new()
        .verbosity(args.log_level)
        .color(ColorChoice::Auto)
        .init()
        .expect("Failed to initialize stderrlog");

    let reader = get_input_reader(&args.input).unwrap();
    let writer = get_output_writer(&args.output).unwrap();
    let geometries = read_geometries(reader, &args.input_format); // lazily loaded
    let transformed = affine_transform(geometries, &args);

    write_geometries(writer, transformed, &args.output_format);
}
