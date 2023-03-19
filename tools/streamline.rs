use std::io::Write;
use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use generative::io::{
    get_input_reader, get_output_writer, read_geometries, write_geometries, GeometryFormat,
};
use geo::{
    AffineOps, AffineTransform, Centroid, Coord, Geometry, Line, LineString, MapCoordsInPlace,
};
use ndarray::Array2;
use rand::distributions::Distribution;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use rand_distr::Binomial;
use stderrlog::ColorChoice;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum StreamlineKind {
    /// One streamline per vertex
    ///
    /// Geometry is non-rigid, and will transform in wonky ways
    PerVertex,
    /// One streamline at the centroid of each geometry
    ///
    /// Treats the geometries as rigid, but doesn't consider the geometries dimensions
    PerCentroid,
}

impl std::fmt::Display for StreamlineKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            // important: Should match clap::ValueEnum format
            StreamlineKind::PerVertex => write!(f, "per-vertex"),
            StreamlineKind::PerCentroid => write!(f, "per-centroid"),
        }
    }
}

/// Generate vector field streamlines for the given geometries
#[derive(Debug, Parser)]
#[clap(name = "streamline", verbatim_doc_comment)]
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

    /// Output geometry format.
    #[clap(short = 'O', long, default_value_t = GeometryFormat::Wkt)]
    pub output_format: GeometryFormat,

    /// The function fn f(x: f64, y: f64) -> [f64; 2] that defines the vector field.
    ///
    /// If not given, a Perlin noise field will be used instead.
    ///
    /// TODO: Use rhai to evaluate a function
    /// TODO: Use Perlin noise
    #[clap(short, long)]
    pub function: Option<String>,

    /// The random seed to use. Use zero to let the tool pick its own random seed.
    #[clap(long, default_value_t = 0)]
    pub seed: u64,

    /// The minimum x coordinate of the vector field
    #[clap(short = 'x', long, default_value_t = 0.0)]
    pub min_x: f64,

    /// The maximum x coordinate of the vector field
    #[clap(short = 'X', long, default_value_t = 20.0)]
    pub max_x: f64,

    /// The minimum y coordinate of the vector field
    #[clap(short = 'y', long, default_value_t = 0.0)]
    pub min_y: f64,

    /// The maximum y coordinate of the vector field
    #[clap(short = 'Y', long, default_value_t = 20.0)]
    pub max_y: f64,

    /// The vector field grid spacing
    #[clap(short = 'd', long, default_value_t = 0.5)]
    pub delta_h: f64,

    /// The size of each time step
    #[clap(short = 't', long, default_value_t = 0.1)]
    pub delta_t: f64,

    /// The number of time steps to make
    #[clap(short = 'T', long, default_value_t = 10)]
    pub time_steps: usize,

    /// Whether to make the number of timesteps random (with mean '--time_steps') for each input geometry.
    #[clap(short = 'r', long)]
    pub random_timesteps: bool,

    /// Draw the vector field
    #[clap(short = 'v', long)]
    pub draw_vector_field: bool,

    /// WKT-like SVG styles to apply to the vector field
    #[clap(short = 'V', long)]
    pub vector_field_style: Option<String>,

    /// The kind of streamlines to draw for each geometry
    #[clap(short = 'k', long, default_value_t = StreamlineKind::PerCentroid)]
    pub streamline_kind: StreamlineKind,

    /// Disable drawing streamlines
    #[clap(short = 'n', long)]
    pub no_draw_streamlines: bool,

    /// WKT-like SVG styles to apply to the streamlines
    #[clap(short = 'S', long)]
    pub streamline_style: Option<String>,

    /// Draw the geometries after simulation
    #[clap(short = 'g', long)]
    pub draw_geometries: bool,

    /// WKT-like SVG styles to apply to the geometries
    #[clap(short = 'G', long)]
    pub geometry_style: Option<String>,
}

fn generate_random_seed_if_not_specified(seed: u64) -> u64 {
    if seed == 0 {
        let mut rng = rand::thread_rng();
        rng.gen()
    } else {
        seed
    }
}

// TODO: How to handle poles?
fn default_field(x: f64, y: f64) -> [f64; 2] {
    let temp = f64::sqrt(x.powi(2) + y.powi(2) + 4.0);
    [-x / temp, y / temp]
}

#[derive(Debug, Clone, PartialEq)]
struct VectorField {
    field: Array2<[f64; 2]>,

    min_x: f64,
    max_x: f64,
    min_y: f64,
    max_y: f64,
    stride: f64,
}

impl VectorField {
    fn new(min_x: f64, max_x: f64, min_y: f64, max_y: f64, stride: f64) -> Self {
        let max_i = (max_x - min_x) / stride;
        let max_i = max_i as usize;
        let max_j = (max_y - min_y) / stride;
        let max_j = max_j as usize;
        Self {
            field: Array2::from_elem((max_i, max_j), [0.0, 0.0]),
            min_x,
            max_x,
            min_y,
            max_y,
            stride,
        }
    }

    fn evaluate<F>(&mut self, func: F)
    where
        F: Fn(f64, f64) -> [f64; 2],
    {
        for ((i, j), val) in self.field.indexed_iter_mut() {
            let x = (i as f64) * self.stride + self.min_x;
            let y = (j as f64) * self.stride + self.min_y;
            *val = func(x, y);
        }
    }

    fn write<W>(&self, writer: &mut W, format: &GeometryFormat)
    where
        W: std::io::Write,
    {
        let geoms = self.field.indexed_iter().map(|((i, j), val)| {
            let x1 = (i as f64) * self.stride + self.min_x;
            let y1 = (j as f64) * self.stride + self.min_y;

            // Vector field visualizations don't look good if the vectors use the same scale as the
            // uniform grid they're drawn on. So we scale by the delta-h.
            let dx = val[0] * self.stride;
            let dy = val[1] * self.stride;

            let x2 = x1 + dx;
            let y2 = y1 + dy;

            let line = Line::new(Coord { x: x1, y: y1 }, Coord { x: x2, y: y2 });
            Geometry::Line(line)
        });

        write_geometries(writer, geoms, format);
    }
}

#[allow(clippy::too_many_arguments)]
fn simulate<'v, G>(
    geometries: G,
    vector_field: &'v VectorField,
    timestep: f64,
    num_timesteps: usize,
    random_timesteps: bool,
    rng: &'v mut StdRng,
    streamline_kind: StreamlineKind,
    record_streamlines: bool,
) -> impl Iterator<Item = (Geometry, Vec<LineString>)> + 'v
where
    G: IntoIterator<Item = Geometry>,
    G: 'v,
{
    geometries.into_iter().map(move |g| {
        let num_timesteps = if random_timesteps {
            let n = num_timesteps * 2; // changes mean
            let p = 0.5; // changes skew
            let dist = Binomial::new(n as u64, p).unwrap();
            dist.sample(rng) as usize
        } else {
            num_timesteps
        };
        simulate_geometry(
            g,
            vector_field,
            timestep,
            num_timesteps,
            streamline_kind,
            record_streamlines,
        )
    })
}

fn simulate_geometry(
    geometry: Geometry,
    vector_field: &VectorField,
    timestep: f64,
    num_timesteps: usize,
    streamline_kind: StreamlineKind,
    record_streamlines: bool,
) -> (Geometry, Vec<LineString>) {
    match streamline_kind {
        StreamlineKind::PerVertex => simulate_geom_vertices(
            geometry,
            vector_field,
            timestep,
            num_timesteps,
            record_streamlines,
        ),
        StreamlineKind::PerCentroid => {
            let (geom, single_streamline) = simulate_rigid_geometry(
                geometry,
                vector_field,
                timestep,
                num_timesteps,
                record_streamlines,
            );
            (geom, vec![single_streamline])
        }
    }
}

fn simulate_coordinate(
    original: Coord,
    field: &VectorField,
    timestep: f64,
    num_timesteps: usize,
    record_streamlines: bool,
) -> (AffineTransform, LineString) {
    let mut streamline = Vec::with_capacity(if record_streamlines { num_timesteps } else { 0 });
    let mut current = original;
    if record_streamlines {
        streamline.push(current);
    }
    for _ in 0..num_timesteps {
        if current.x > field.max_x || current.y > field.max_y {
            break;
        }
        let i = ((current.x - field.min_x) / field.stride) as usize;
        let j = ((current.y - field.min_y) / field.stride) as usize;

        let current_vector = field.field[[i, j]];

        current.x += timestep * current_vector[0];
        current.y += timestep * current_vector[1];

        if record_streamlines {
            streamline.push(current);
        }
    }

    // TODO: Keep track of the initial and final coordinate orientations
    let offset = current - original;
    let transform = AffineTransform::translate(offset.x, offset.y);

    (transform, LineString::new(streamline))
}

fn simulate_rigid_geometry(
    mut geometry: Geometry,
    vector_field: &VectorField,
    timestep: f64,
    num_timesteps: usize,
    record_streamlines: bool,
) -> (Geometry, LineString) {
    match geometry.centroid() {
        Some(centroid) => {
            let (transform, streamline) = simulate_coordinate(
                centroid.into(),
                vector_field,
                timestep,
                num_timesteps,
                record_streamlines,
            );

            geometry.affine_transform_mut(&transform);

            (geometry, streamline)
        }
        // I'm not actually sure when finding the centroid would fail, unless the geometry is
        // empty.
        None => (geometry, LineString::new(vec![])),
    }
}

fn simulate_geom_vertices(
    mut geometry: Geometry,
    vector_field: &VectorField,
    timestep: f64,
    num_timesteps: usize,
    record_streamlines: bool,
) -> (Geometry, Vec<LineString>) {
    let streamlines = vec![];
    geometry.map_coords_in_place(|coord| {
        let (transform, _streamline) = simulate_coordinate(
            coord,
            vector_field,
            timestep,
            num_timesteps,
            record_streamlines,
        );
        // TODO: All of the Geometry iteration traits don't allow for side-effects.
        // map_coords_in_place requires the Fn be Copy, which precludes sharing the mutable
        // streamlines reference. try_map_coords_in_place doesn't have Copy, but is still Fn, which
        // also precludes sharing a mutable reference.
        //
        // streamlines.push(streamline);
        transform.apply(coord)
    });

    (geometry, streamlines)
}

fn main() {
    let args = CmdlineOptions::parse();

    stderrlog::new()
        .verbosity(args.log_level)
        .color(ColorChoice::Auto)
        .init()
        .expect("Failed to initialize stderrlog");

    let seed = generate_random_seed_if_not_specified(args.seed);
    log::info!("Seeding RNG with: {}", seed);
    let mut rng = StdRng::seed_from_u64(seed);

    let mut field = VectorField::new(args.min_x, args.max_x, args.min_y, args.max_y, args.delta_h);
    // TODO: It shouldn't actually be necessary to pre-evaluate the whole field. I can use a
    // continuous vector field, and if --draw-vector-field is given, _then_ discretize it.
    log::info!("Evaluating vector field...");
    field.evaluate(default_field);

    let reader = get_input_reader(&args.input).unwrap();
    let mut writer = get_output_writer(&args.output).unwrap();

    if args.draw_vector_field {
        // TODO: This doesn't work if vector_field_style contains multiple space-separated styles.
        // One possible solution would be to make the wkt2svg STYLE parser a proper parser.
        if let Some(style) = args.vector_field_style {
            writeln!(&mut writer, "{style}").unwrap();
        }
        field.write(&mut writer, &args.output_format);
    } else {
        log::debug!("{field:.4?}");
    }

    let geometries = read_geometries(reader, &args.input_format);

    let geoms_and_streamlines = simulate(
        geometries,
        &field,
        args.delta_t,
        args.time_steps,
        args.random_timesteps,
        &mut rng,
        args.streamline_kind,
        !args.no_draw_streamlines,
    );
    let (geometries, streamlines): (Vec<_>, Vec<_>) = geoms_and_streamlines.unzip();
    let streamlines = streamlines.into_iter().flatten().map(Geometry::LineString);

    if let Some(style) = args.streamline_style {
        writeln!(&mut writer, "{style}").unwrap();
    }
    write_geometries(&mut writer, streamlines, &args.output_format);
    if let Some(style) = args.geometry_style {
        writeln!(&mut writer, "{style}").unwrap();
    }
    write_geometries(&mut writer, geometries, &args.output_format);
}