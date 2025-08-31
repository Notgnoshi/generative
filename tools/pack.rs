use std::collections::BTreeMap;
use std::path::PathBuf;

use clap::Parser;
use generative::io::{
    GeometryFormat, get_input_reader, get_output_writer, read_geometries, write_geometries,
};
use geo::{BoundingRect, Coord, Translate};
use rectangle_pack::{
    GroupedRectsToPlace, RectToInsert, TargetBin, contains_smallest_box, pack_rects,
    volume_heuristic,
};

/// Pack the given geometries into a rectangle
///
/// Each geometry is packed separately, so if you have a more complex arrangement, use the 'bundle'
/// tool to bundle the geometries into a single GEOMETRYCOLLECTION.
#[derive(Debug, Parser)]
#[clap(name = "pack", verbatim_doc_comment)]
struct CmdlineOptions {
    /// The log level
    #[clap(short, long, default_value_t = tracing::Level::INFO)]
    log_level: tracing::Level,

    /// Input file to read input from. Defaults to stdin.
    #[clap(short, long)]
    input: Option<PathBuf>,

    /// Input geometry format.
    #[clap(short = 'I', long, default_value_t = GeometryFormat::Wkt)]
    input_format: GeometryFormat,

    /// Output file to write result to. Defaults to stdout.
    #[clap(short, long)]
    output: Option<PathBuf>,

    /// Output geometry format.
    #[clap(short = 'O', long, default_value_t = GeometryFormat::Wkt)]
    output_format: GeometryFormat,

    /// The width of the target rectangle
    #[clap(long, default_value_t = 100)]
    width: u32,

    /// The height of the target rectangle
    #[clap(long, default_value_t = 100)]
    height: u32,

    /// Padding to apply around each geometry's bounding box
    #[clap(long, default_value_t = 0.5)]
    padding: f64,
}

fn main() -> Result<(), String> {
    let args = CmdlineOptions::parse();

    let filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(args.log_level.into())
        .from_env_lossy();
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_ansi(true)
        .with_writer(std::io::stderr)
        .init();

    let reader = get_input_reader(&args.input).unwrap();
    let mut geometries: Vec<_> = read_geometries(reader, &args.input_format).collect();
    let padding = Coord {
        x: args.padding,
        y: args.padding,
    };

    let mut rects = GroupedRectsToPlace::<usize, ()>::new();
    for (idx, geometry) in geometries.iter().enumerate() {
        if let Some(mut bbox) = geometry.bounding_rect() {
            // Apply a little bit of padding around each bounding box. This is not just to generate
            // a more appealing result; it's so that POINTs don't get "packed" all on top of each
            // other because they have 0x0 bounding boxes.
            let min = bbox.min();
            let new_min = min - padding;
            bbox.set_min(new_min);

            let max = bbox.max();
            let new_max = max + padding;
            bbox.set_max(new_max);
            let rect =
                RectToInsert::new(bbox.width().ceil() as u32, bbox.height().ceil() as u32, 1);
            rects.push_rect(idx, None, rect);
        }
    }

    let mut bins = BTreeMap::new();
    bins.insert(0, TargetBin::new(args.width, args.height, 1));

    match pack_rects(&rects, &mut bins, &volume_heuristic, &contains_smallest_box) {
        Ok(packing) => {
            let packing = packing.packed_locations();
            for (idx, (_bin, location)) in packing.iter() {
                let geometry = &mut geometries[*idx];
                let bbox = geometry.bounding_rect().unwrap();
                let source = bbox.center();
                let target = Coord {
                    x: location.x() as f64,
                    y: location.y() as f64,
                };
                let offset = target - source;
                geometry.translate_mut(offset.x, offset.y);
            }
        }
        Err(e) => {
            return Err(format!(
                "Failed to pack geometries into the given {}x{} rectangle: {e}",
                args.width, args.height
            ));
        }
    }

    let writer = get_output_writer(&args.output).unwrap();
    write_geometries(writer, geometries, args.output_format);
    Ok(())
}
