use std::path::PathBuf;

use clap::{ArgGroup, Parser};
use generative::flatten::flatten_nested_geometries;
use generative::io::{get_input_reader, get_output_writer, read_geometries, GeometryFormat};
use geo::{
    AffineOps, AffineTransform, BoundingRect, Coord, CoordsIter, Geometry, Line, LineString, Point,
    Polygon, Rect, Triangle,
};
use stderrlog::ColorChoice;
use svg::node::element;
use svg::Document;

/// Convert the given geometries to SVG
///
/// Examples:
///     ... | wkt2svg | display -density 500 -
///     ... | wkt2svg --output /tmp/geometries.svg
#[derive(Debug, Parser)]
#[clap(name = "wkt2svg", verbatim_doc_comment)]
#[clap(group(ArgGroup::new("sizing")))]
pub struct CmdlineOptions {
    /// The log level
    #[clap(short, long, default_value_t = log::Level::Info)]
    pub log_level: log::Level,

    /// Input file to read input from. Defaults to stdin.
    #[clap(short, long)]
    pub input: Option<PathBuf>,

    /// Input geometry format.
    #[clap(short = 'I', long, default_value_t = GeometryFormat::Wkt)]
    pub input_format: GeometryFormat,

    /// Output file to write result to. Defaults to stdout.
    #[clap(short, long)]
    pub output: Option<PathBuf>,

    /// Scale the geometries the specified amount, and then scale the viewbox to fit.
    ///
    /// Mutually exclusive with --viewbox
    #[clap(short, long, group = "sizing")]
    pub scale: Option<f64>,

    /// Scale the geometries to fit the given (x1, y1, x2 y2) viewbox
    ///
    /// Mutually exclusive with --scale
    #[clap(short, long, group = "sizing", number_of_values = 4)]
    pub viewbox: Option<Vec<f64>>,

    /// Whether to add a little bit of padding all the way around the viewbox
    #[clap(short, long, default_value_t = false)]
    pub padding: bool,
    // TODO: Global styling
}

enum ScaleType {
    Identity,
    ScaleAndExpandViewBoxToFit(f64),
    FitToViewBox(Rect),
}

impl From<&CmdlineOptions> for ScaleType {
    fn from(options: &CmdlineOptions) -> Self {
        match (options.scale, options.viewbox.as_ref()) {
            (None, None) => Self::Identity,
            (Some(scale), None) => Self::ScaleAndExpandViewBoxToFit(scale),
            // The viewbox array is guaranteed to be 4 elements long by clap
            (None, Some(v)) => {
                let viewbox = Rect::new(Coord { x: v[0], y: v[1] }, Coord { x: v[2], y: v[3] });
                Self::FitToViewBox(viewbox)
            }
            _ => unreachable!("--scale and --viewbox are mutually exclusive, as enforced by clap"),
        }
    }
}

struct SvgOptions {
    scale_type: ScaleType,
    // TODO: styling
}

impl From<&CmdlineOptions> for SvgOptions {
    fn from(options: &CmdlineOptions) -> Self {
        Self {
            scale_type: ScaleType::from(options),
        }
    }
}

fn add_point_to_document(point: Point, document: Document, _options: &SvgOptions) -> Document {
    let circle = element::Circle::new()
        .set("cx", point.x())
        .set("cy", point.y())
        .set("r", 1.0);
    document.add(circle)
}

fn add_line_to_document(line: Line, document: Document, _options: &SvgOptions) -> Document {
    let line = element::Line::new()
        .set("x1", line.start.x)
        .set("y1", line.start.y)
        .set("x2", line.end.x)
        .set("y2", line.end.y);
    document.add(line)
}

fn add_linestring_to_document(
    linestring: LineString,
    document: Document,
    _options: &SvgOptions,
) -> Document {
    let points: Vec<(f64, f64)> = linestring
        .into_inner()
        .into_iter()
        .map(|c| c.into())
        .collect();
    let polyline = element::Polyline::new().set("points", points);
    document.add(polyline)
}

fn add_ring_to_path_data(mut data: element::path::Data, ring: &LineString) -> element::path::Data {
    if ring.coords_count() < 3 {
        return data;
    }

    // Handle both closed and open rings
    let end = if ring.0.first() == ring.0.last() {
        ring.0.len() - 1
    } else {
        ring.0.len()
    };
    // Orientation doesn't matter in SVG path data.
    let start: (f64, f64) = ring.0[0].into();
    data = data.move_to(start);
    for coord in &ring.0[1..end] {
        let point = (coord.x, coord.y);
        data = data.line_to(point);
    }
    data.close()
}

fn add_polygon_to_document(
    polygon: Polygon,
    document: Document,
    _options: &SvgOptions,
) -> Document {
    let mut data = element::path::Data::new();
    data = add_ring_to_path_data(data, polygon.exterior());
    for interior in polygon.interiors() {
        data = add_ring_to_path_data(data, interior);
    }
    let path = element::Path::new()
        .set("fill-rule", "evenodd")
        .set("d", data);

    document.add(path)
}

fn add_rect_to_document(rect: Rect, document: Document, _options: &SvgOptions) -> Document {
    let rect = element::Rectangle::new()
        .set("x", rect.min().x)
        .set("y", rect.max().y) // (x, y) is upper left corner
        .set("width", rect.width())
        .set("height", rect.height());
    document.add(rect)
}

fn add_triangle_to_document(
    triangle: Triangle,
    document: Document,
    _options: &SvgOptions,
) -> Document {
    let points: Vec<(f64, f64)> = triangle.to_array().into_iter().map(|c| c.into()).collect();
    let polygon = element::Polygon::new().set("points", points);
    document.add(polygon)
}

fn to_svg(
    geometry: Geometry,
    transform: &Option<AffineTransform>,
    document: Document,
    options: &SvgOptions,
) -> Document {
    let transformed_geometry = if let Some(transform) = transform {
        geometry.affine_transform(transform)
    } else {
        geometry
    };

    match transformed_geometry {
        Geometry::Point(p) => add_point_to_document(p, document, options),
        Geometry::Line(l) => add_line_to_document(l, document, options),
        Geometry::LineString(l) => add_linestring_to_document(l, document, options),
        Geometry::Polygon(p) => add_polygon_to_document(p, document, options),
        Geometry::Rect(r) => add_rect_to_document(r, document, options),
        Geometry::Triangle(t) => add_triangle_to_document(t, document, options),
        _ => unreachable!("MULTI-geometries get flattened before conversion to SVG"),
    }
}

fn expand_to_fit(bbox1: Option<Rect>, bbox2: Option<Rect>) -> Option<Rect> {
    match (bbox1, bbox2) {
        (None, None) => None,
        (Some(b), None) | (None, Some(b)) => Some(b),
        (Some(bbox1), Some(bbox2)) => {
            let min1 = bbox1.min();
            let min2 = bbox2.min();
            let min = Coord {
                x: f64::min(min1.x, min2.x),
                y: f64::min(min1.y, min2.y),
            };

            let max1 = bbox1.max();
            let max2 = bbox2.max();
            let max = Coord {
                x: f64::max(max1.x, max2.x),
                y: f64::max(max1.y, max2.y),
            };

            Some(Rect::new(min, max))
        }
    }
}

fn bounding_box<'g>(geometries: impl Iterator<Item = &'g Geometry>) -> Option<Rect> {
    let mut bbox = None;
    for geometry in geometries {
        let temp = geometry.bounding_rect();
        bbox = expand_to_fit(bbox, temp);
    }
    bbox
}

fn calculate_transform(
    bounding_box: &Rect,
    options: &SvgOptions,
) -> (Option<AffineTransform>, Rect) {
    match options.scale_type {
        ScaleType::Identity => (None, *bounding_box),
        ScaleType::ScaleAndExpandViewBoxToFit(scale) => {
            let center = bounding_box.center();
            let transform = AffineTransform::scale(scale, scale, center);
            let viewbox = bounding_box.affine_transform(&transform);
            (Some(transform), viewbox)
        }
        ScaleType::FitToViewBox(viewbox) => {
            let offset = viewbox.center() - bounding_box.center();
            let x_scale = viewbox.width() / bounding_box.width();
            let y_scale = viewbox.height() / bounding_box.height();
            let center = bounding_box.center();

            let transform =
                AffineTransform::translate(offset.x, offset.y).scaled(x_scale, y_scale, center);

            (Some(transform), viewbox)
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
    // Can't lazily convert to SVG because we have to know the whole collection's bounding box to
    // know how to scale.
    let geometries = read_geometries(reader, &args.input_format);
    let geometries: Vec<_> = flatten_nested_geometries(geometries).collect();
    if geometries.is_empty() {
        return;
    }
    let bbox = bounding_box(geometries.iter());
    if bbox.is_none() {
        log::error!("Failed to calculate geometry bounding box");
        return;
    }
    let bbox = bbox.unwrap();
    let options = SvgOptions::from(&args);

    let (transform, mut viewbox) = calculate_transform(&bbox, &options);
    if args.padding {
        const PADDING: Coord = Coord { x: 3.0, y: 3.0 };
        let min = viewbox.min();
        let new_min = min - PADDING;
        viewbox.set_min(new_min);

        let max = viewbox.max();
        let new_max = max + PADDING;
        viewbox.set_max(new_max);
    }
    let min = viewbox.min();
    let viewbox = (min.x, min.y, viewbox.width(), viewbox.height());
    log::debug!(
        "Transforming geometries with: {:?} to fit into viewBox {:?}",
        transform,
        viewbox
    );

    let mut document = Document::new().set("viewBox", viewbox);
    let style = element::Style::new("svg { stroke:black; stroke-width:2px; fill:none;}");
    document = document.add(style);
    for geometry in geometries {
        document = to_svg(geometry, &transform, document, &options);
    }

    let writer = get_output_writer(&args.output).unwrap();
    svg::write(writer, &document).unwrap();
}
