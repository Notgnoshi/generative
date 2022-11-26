mod cmdline;
use cmdline::{CmdlineOptions, TransformCenter};

use clap::Parser;
use generative::stdio::{get_input_reader, get_output_writer};
use generative::wkio::{read_geometries, write_geometries};
use geo::{coord, AffineOps, AffineTransform, BoundingRect, Coord, Geometry, Rect};
use stderrlog::ColorChoice;
use wkt::ToWkt;

fn build_transform(args: &CmdlineOptions, center: Coord) -> AffineTransform {
    let mut transform = AffineTransform::rotate(args.rotation, center);
    if let Some(scale) = &args.scale {
        // The clap parser guarantees that the Vec<f64> used for scale, offset, and skew have
        // exactly 2 values.
        transform = transform.scaled(scale[0], scale[1], center);
    }
    if let Some(offset) = &args.offset {
        transform = transform.translated(offset[0], offset[1]);
    }
    if let Some(skew) = &args.skew {
        transform = transform.skewed(skew[0], skew[1], center);
    }

    transform
}

fn main() {
    let args = cmdline::CmdlineOptions::parse();

    stderrlog::new()
        .verbosity(args.verbosity as usize + 1) // Default to WARN level.
        .color(ColorChoice::Auto)
        .init()
        .expect("Failed to initialize stderrlog");

    let reader = get_input_reader(&args.input).unwrap();
    let writer = get_output_writer(&args.output).unwrap();
    let geometries = read_geometries(reader, &args.input_format); // lazily loaded

    match args.center {
        TransformCenter::Origin => {
            let center = coord! {x:0.0, y: 0.0};
            let transform = build_transform(&args, center);
            let transformed = geometries.map(|geom| geom.affine_transform(&transform));
            write_geometries(writer, transformed, &args.output_format);
        }
        TransformCenter::EachGeometry => {
            let transformed = geometries.map(|geom| {
                let center = geom
                    .bounding_rect()
                    .unwrap_or_else(|| {
                        panic!(
                            "Geometry '{}' didn't have a bounding rectangle",
                            geom.to_wkt()
                        )
                    })
                    .center();
                let transform = build_transform(&args, center);
                geom.affine_transform(&transform)
            });
            write_geometries(writer, transformed, &args.output_format);
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
            let transform = build_transform(&args, center);

            // Instead of applying the transformation in-place all at once _and then_ writing the
            // results, we lazily perform the transformation so that we can pipeline the
            // transformation and the serialization.
            let transformed = geometries
                .into_iter()
                .map(|geom| geom.affine_transform(&transform));
            write_geometries(writer, transformed, &args.output_format);
        }
    }
}
