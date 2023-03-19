use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use generative::io::{
    get_input_reader, get_output_writer, read_geometries, write_geometries, GeometryFormat,
};
use ndarray::Array2;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use stderrlog::ColorChoice;

#[derive(Debug, Clone, ValueEnum)]
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

    /// The function f(x, y) -> (x, y) that defines the vector field.
    ///
    /// If not given, a Perlin noise field will be used instead.
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

    /// Draw a streamline for each geometry
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
    _max_x: f64,
    min_y: f64,
    _max_y: f64,
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
            _max_x: max_x,
            min_y,
            _max_y: max_y,
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
    let _rng = StdRng::seed_from_u64(seed);

    log::info!("Evaluating vector field...");
    let mut field = VectorField::new(args.min_x, args.max_x, args.min_y, args.max_y, args.delta_h);
    field.evaluate(default_field);
    log::debug!("{field:.4?}");

    let reader = get_input_reader(&args.input).unwrap();
    let geometries = read_geometries(reader, &args.input_format);

    // Do some kind of transformation to the geometries here.

    let writer = get_output_writer(&args.output).unwrap();
    write_geometries(writer, geometries, &args.output_format);
}
