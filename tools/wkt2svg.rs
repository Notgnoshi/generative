use std::path::PathBuf;

use clap::{ArgGroup, Parser};
use generative::flatten::flatten_nested_geometries;
use generative::io::{
    get_input_reader, get_output_writer, read_wkt_geometries_and_styles, GeometryAndStyle, SvgStyle,
};
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

    #[clap(long, default_value_t = 1.0)]
    pub point_radius: f64,

    #[clap(long, default_value = "black")]
    pub stroke: String,

    #[clap(long, default_value_t = 2.0)]
    pub stroke_width: f64,

    #[clap(long)]
    pub stroke_dasharray: Option<String>,

    #[clap(long, default_value = "none")]
    pub fill: String,
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
    global_point_radius: f64,
    overridden_point_radius: Option<f64>,
    global_stroke: String,
    overridden_stroke: Option<String>,
    global_stroke_width: f64,
    overridden_stroke_width: Option<f64>,
    // This is the only global setting that may or may not be present
    global_stroke_dasharray: Option<String>,
    overridden_stroke_dasharray: Option<String>,
    global_fill: String,
    overridden_fill: Option<String>,
}

impl From<&CmdlineOptions> for SvgOptions {
    fn from(options: &CmdlineOptions) -> Self {
        Self {
            scale_type: ScaleType::from(options),
            global_point_radius: options.point_radius,
            overridden_point_radius: None,
            global_stroke: options.stroke.clone(),
            overridden_stroke: None,
            global_stroke_width: options.stroke_width,
            overridden_stroke_width: None,
            global_stroke_dasharray: options.stroke_dasharray.clone(),
            overridden_stroke_dasharray: None,
            global_fill: options.fill.clone(),
            overridden_fill: None,
        }
    }
}

impl SvgOptions {
    fn get_global_style(&self) -> element::Style {
        // let style = element::Style::new("svg { stroke:black; stroke-width:2px; fill:none;}");

        let style = if let Some(dasharray) = &self.global_stroke_dasharray {
            format!("stroke-dasharray:{dasharray};")
        } else {
            String::new()
        };

        let style = format!(
            "{} stroke:{}; stroke-width:{}; fill:{};",
            style, self.global_stroke, self.global_stroke_width, self.global_fill
        );

        element::Style::new(format!("svg {{{style}}}"))
    }
}

fn add_point_to_document(point: Point, document: Document, options: &SvgOptions) -> Document {
    let radius = if let Some(radius) = options.overridden_point_radius {
        radius
    } else {
        options.global_point_radius
    };
    let mut node = element::Circle::new()
        .set("cx", point.x())
        .set("cy", point.y())
        .set("r", radius);

    // This is an unfortunate bit of copy-pasta, but the `.set()` method doesn't come from a trait,
    // so it's more difficult to make generic than I wanted.
    if let Some(stroke) = options.overridden_stroke.as_ref() {
        node = node.set("stroke", stroke.clone());
    }
    if let Some(width) = options.overridden_stroke_width.as_ref() {
        node = node.set("stroke-width", *width);
    }
    if let Some(dasharray) = options.overridden_stroke_dasharray.as_ref() {
        node = node.set("stroke-dasharray", dasharray.clone());
    }
    if let Some(fill) = options.overridden_fill.as_ref() {
        node = node.set("fill", fill.clone());
    }
    document.add(node)
}

fn add_line_to_document(line: Line, document: Document, options: &SvgOptions) -> Document {
    let mut node = element::Line::new()
        .set("x1", line.start.x)
        .set("y1", line.start.y)
        .set("x2", line.end.x)
        .set("y2", line.end.y);

    if let Some(stroke) = options.overridden_stroke.as_ref() {
        node = node.set("stroke", stroke.clone());
    }
    if let Some(width) = options.overridden_stroke_width.as_ref() {
        node = node.set("stroke-width", *width);
    }
    if let Some(dasharray) = options.overridden_stroke_dasharray.as_ref() {
        node = node.set("stroke-dasharray", dasharray.clone());
    }
    if let Some(fill) = options.overridden_fill.as_ref() {
        node = node.set("fill", fill.clone());
    }
    document.add(node)
}

fn add_linestring_to_document(
    linestring: LineString,
    document: Document,
    options: &SvgOptions,
) -> Document {
    let points: Vec<(f64, f64)> = linestring
        .into_inner()
        .into_iter()
        .map(|c| c.into())
        .collect();
    let mut node = element::Polyline::new().set("points", points);

    if let Some(stroke) = options.overridden_stroke.as_ref() {
        node = node.set("stroke", stroke.clone());
    }
    if let Some(width) = options.overridden_stroke_width.as_ref() {
        node = node.set("stroke-width", *width);
    }
    if let Some(dasharray) = options.overridden_stroke_dasharray.as_ref() {
        node = node.set("stroke-dasharray", dasharray.clone());
    }
    if let Some(fill) = options.overridden_fill.as_ref() {
        node = node.set("fill", fill.clone());
    }
    document.add(node)
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

fn add_polygon_to_document(polygon: Polygon, document: Document, options: &SvgOptions) -> Document {
    let mut data = element::path::Data::new();
    data = add_ring_to_path_data(data, polygon.exterior());
    for interior in polygon.interiors() {
        data = add_ring_to_path_data(data, interior);
    }
    let mut node = element::Path::new()
        .set("fill-rule", "evenodd")
        .set("d", data);

    if let Some(stroke) = options.overridden_stroke.as_ref() {
        node = node.set("stroke", stroke.clone());
    }
    if let Some(width) = options.overridden_stroke_width.as_ref() {
        node = node.set("stroke-width", *width);
    }
    if let Some(dasharray) = options.overridden_stroke_dasharray.as_ref() {
        node = node.set("stroke-dasharray", dasharray.clone());
    }
    if let Some(fill) = options.overridden_fill.as_ref() {
        node = node.set("fill", fill.clone());
    }
    document.add(node)
}

fn add_rect_to_document(rect: Rect, document: Document, options: &SvgOptions) -> Document {
    let mut node = element::Rectangle::new()
        .set("x", rect.min().x)
        .set("y", rect.max().y) // (x, y) is upper left corner
        .set("width", rect.width())
        .set("height", rect.height());

    if let Some(stroke) = options.overridden_stroke.as_ref() {
        node = node.set("stroke", stroke.clone());
    }
    if let Some(width) = options.overridden_stroke_width.as_ref() {
        node = node.set("stroke-width", *width);
    }
    if let Some(dasharray) = options.overridden_stroke_dasharray.as_ref() {
        node = node.set("stroke-dasharray", dasharray.clone());
    }
    if let Some(fill) = options.overridden_fill.as_ref() {
        node = node.set("fill", fill.clone());
    }
    document.add(node)
}

fn add_triangle_to_document(
    triangle: Triangle,
    document: Document,
    options: &SvgOptions,
) -> Document {
    let points: Vec<(f64, f64)> = triangle.to_array().into_iter().map(|c| c.into()).collect();
    let mut node = element::Polygon::new().set("points", points);

    if let Some(stroke) = options.overridden_stroke.as_ref() {
        node = node.set("stroke", stroke.clone());
    }
    if let Some(width) = options.overridden_stroke_width.as_ref() {
        node = node.set("stroke-width", *width);
    }
    if let Some(dasharray) = options.overridden_stroke_dasharray.as_ref() {
        node = node.set("stroke-dasharray", dasharray.clone());
    }
    if let Some(fill) = options.overridden_fill.as_ref() {
        node = node.set("fill", fill.clone());
    }
    document.add(node)
}

fn to_svg(
    geometry: GeometryAndStyle,
    transform: &Option<AffineTransform>,
    document: Document,
    options: &mut SvgOptions,
) -> Document {
    match geometry {
        GeometryAndStyle::Style(style) => {
            match style {
                SvgStyle::PointRadius(r) => {
                    if r == options.global_point_radius {
                        options.overridden_point_radius = None;
                    } else {
                        options.overridden_point_radius = Some(r);
                    }
                }
                SvgStyle::Stroke(s) => {
                    if s == options.global_stroke {
                        options.overridden_stroke = None;
                    } else {
                        options.overridden_stroke = Some(s);
                    }
                }
                SvgStyle::StrokeWidth(w) => {
                    if w == options.global_stroke_width {
                        options.overridden_stroke_width = None;
                    } else {
                        options.overridden_stroke_width = Some(w);
                    }
                }
                SvgStyle::StrokeDasharray(d) => {
                    if Some(&d) == options.global_stroke_dasharray.as_ref() {
                        options.overridden_stroke_dasharray = None;
                    } else {
                        options.overridden_stroke_dasharray = Some(d);
                    }
                }
                SvgStyle::Fill(f) => {
                    if f == options.global_fill {
                        options.overridden_fill = None;
                    } else {
                        options.overridden_fill = Some(f);
                    }
                }
            }
            document
        }
        _ => {
            let geometry: Geometry = geometry.into();
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

fn bounding_box<'g>(geometries: impl Iterator<Item = &'g GeometryAndStyle>) -> Option<Rect> {
    let mut bbox = None;
    for geometry in geometries {
        let temp = match geometry {
            GeometryAndStyle::Point(p) => Some(p.bounding_rect()),
            GeometryAndStyle::Line(l) => Some(l.bounding_rect()),
            GeometryAndStyle::LineString(l) => l.bounding_rect(),
            GeometryAndStyle::Polygon(p) => p.bounding_rect(),
            GeometryAndStyle::MultiPoint(m) => m.bounding_rect(),
            GeometryAndStyle::MultiLineString(m) => m.bounding_rect(),
            GeometryAndStyle::MultiPolygon(m) => m.bounding_rect(),
            GeometryAndStyle::GeometryCollection(g) => g.bounding_rect(),
            GeometryAndStyle::Rect(r) => Some(r.bounding_rect()),
            GeometryAndStyle::Triangle(t) => Some(t.bounding_rect()),
            GeometryAndStyle::Style(_) => None,
        };
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
    let geometries = read_wkt_geometries_and_styles(reader);

    // Flatten any MULTI or GEOMETRYCOLLECTION geometries
    let geometries: Vec<_> = geometries.collect();
    // [g, g, g, s, g, s, s, g] => [g, g, g, s], [g, s], [s], [g]
    let geometries_ending_with_styles =
        geometries.split_inclusive(|x| matches!(x, GeometryAndStyle::Style(_)));
    let flattened = geometries_ending_with_styles.map(|slice| {
        if matches!(slice.last(), Some(GeometryAndStyle::Style(_))) {
            let (style, geometries) = slice.split_last().unwrap();
            // Have to convert to geo_types to flatten
            let flattened = flatten_nested_geometries(geometries.iter().map(|g| g.clone().into()));
            // But then need to convert back to add the style back in
            let flattened = flattened.map(|g| g.into());
            let mut flattened: Vec<GeometryAndStyle> = flattened.collect();
            flattened.push(style.clone());
            flattened
        } else {
            let flattened = flatten_nested_geometries(slice.iter().map(|g| g.clone().into()));
            let flattened = flattened.map(|g| g.into());
            flattened.collect()
        }
    });
    let geometries: Vec<_> = flattened.flatten().collect();

    if geometries.is_empty() {
        return;
    }
    let bbox = bounding_box(geometries.iter());
    if bbox.is_none() {
        log::error!("Failed to calculate geometry bounding box");
        return;
    }
    let bbox = bbox.unwrap();
    let mut options = SvgOptions::from(&args);

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
    let style = options.get_global_style();
    document = document.add(style);
    for geometry in geometries {
        document = to_svg(geometry, &transform, document, &mut options);
    }

    let writer = get_output_writer(&args.output).unwrap();
    svg::write(writer, &document).unwrap();
}
