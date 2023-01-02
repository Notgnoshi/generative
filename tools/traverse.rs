use std::cmp::Ordering;
use std::path::PathBuf;

use clap::Parser;
use generative::graph::GeometryGraph;
use generative::io::{
    get_input_reader, get_output_writer, read_tgf_graph, write_geometries, GeometryFormat,
};
use geo::{Geometry, LineString, Point};
use petgraph::{EdgeType, Undirected};
use rand::distributions::{Distribution, Uniform};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use rand_distr::Binomial;
use stderrlog::ColorChoice;

/// Randomly traverse the given graph.
///
/// May not visit every node.
#[derive(Debug, Parser)]
#[clap(name = "traverse", verbatim_doc_comment)]
pub struct CmdlineOptions {
    /// The log level
    #[clap(long, default_value_t = log::Level::Info)]
    pub log_level: log::Level,

    /// Input file to read input from. Defaults to stdin.
    ///
    /// Input format is expected to be Trivial Graph Format.
    #[clap(short, long)]
    pub input: Option<PathBuf>,

    /// Output file to write result to. Defaults to stdout.
    #[clap(short, long)]
    pub output: Option<PathBuf>,

    /// The output geometry format
    #[clap(short='O', long, default_value_t=GeometryFormat::Wkt)]
    pub output_format: GeometryFormat,

    /// The random seed to use. Use zero to let the tool pick its own random seed.
    #[clap(long, default_value_t = 0)]
    pub seed: u64,

    /// The number of random graph traversals to perform
    #[clap(short = 't', long, default_value_t = 1)]
    pub traversals: u8,

    /// Whether to perform a random number of traverals with mean '--traversals'
    #[clap(short = 'T', long, default_value_t = false)]
    pub random_traversals: bool,

    /// The length of each random traversal
    #[clap(short = 'l', long, default_value_t = 4)]
    pub length: u8,

    /// Whether to make the length of each traversal random with mean '--length'
    #[clap(short = 'L', long, default_value_t = false)]
    pub random_length: bool,

    /// Remove edges after traversing them
    #[clap(short = 'r', long, default_value_t = false)]
    pub remove_after_traverse: bool,

    /// Output untraversed nodes at the end
    #[clap(short = 'u', long, default_value_t = false)]
    pub untraversed: bool,
}

fn generate_random_seed_if_not_specified(seed: u64) -> u64 {
    if seed == 0 {
        let mut rng = rand::thread_rng();
        rng.gen()
    } else {
        seed
    }
}

fn random_traversal<D>(
    length: usize,
    remove_after_traverse: bool,
    graph: &mut GeometryGraph<D>,
    rng: &mut StdRng,
) -> Option<LineString>
where
    D: EdgeType,
{
    if graph.edge_count() == 0 {
        log::warn!("Graph has no edges. Can't do a traversal");
        return None;
    }

    let mut result = Vec::<Point>::with_capacity(length);

    // Pick a random starting point
    let node_dist = Uniform::from(0..graph.node_count());
    let mut start_index = node_dist.sample(rng).into();

    let point = graph[start_index];
    result.push(point);

    let mut buffer = Vec::new();
    for _ in 0..length {
        let neighbors = graph.neighbors(start_index);
        buffer.clear();
        buffer.extend(neighbors); // use extend to avoid repeated alloc/free on every loop

        // Pick the next node to visit
        let next_index = match buffer.len().cmp(&1) {
            Ordering::Greater => {
                let dist = Uniform::from(0..buffer.len());
                let next_index = dist.sample(rng);
                buffer[next_index]
            }
            Ordering::Equal => buffer[0],
            Ordering::Less => break,
        };
        let point = graph[next_index];
        result.push(point);

        // Remove the traversed edge
        if remove_after_traverse {
            let traversed_edge = graph.find_edge(start_index, next_index).unwrap();
            graph.remove_edge(traversed_edge);

            // If removing the traversed edge left behind orphan nodes, remove them too.
            if buffer.len() < 2 {
                graph.remove_node(start_index);
            }
        }
        let neighbors = graph.neighbors(next_index).count();
        if neighbors < 2 {
            if remove_after_traverse {
                graph.remove_node(next_index);
            }
            log::debug!("Hit the end of a connected component - nowhere to go!");
            break;
        }

        start_index = next_index;
    }

    if result.len() > 1 {
        Some(result.into())
    } else {
        None
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
    let mut rng = StdRng::seed_from_u64(seed);

    let reader = get_input_reader(&args.input).unwrap();
    let mut graph: GeometryGraph<Undirected> = read_tgf_graph(reader);

    let mut num_traversals = if args.random_traversals {
        let n = args.traversals * 2; // changes mean
        let p = 0.5; // changes skew
        let dist = Binomial::new(n as u64, p).unwrap();
        dist.sample(&mut rng)
    } else {
        args.traversals as u64
    };
    if num_traversals == 0 {
        num_traversals = 1;
    }
    log::debug!("Making {} traversals", num_traversals);

    let traversals = std::iter::repeat_with(|| {
        let mut length = if args.random_length {
            let n = args.traversals * 2; // changes mean
            let p = 0.5; // changes skew
            let dist = Binomial::new(n as u64, p).unwrap();
            dist.sample(&mut rng)
        } else {
            args.length as u64
        };
        if length < 2 {
            length = 2;
        }
        log::debug!("Making random traversal with length {}", length);
        random_traversal(
            length as usize,
            args.remove_after_traverse,
            &mut graph,
            &mut rng,
        )
    })
    .take(num_traversals as usize)
    .flatten()
    .map(Geometry::LineString);

    let mut writer = get_output_writer(&args.output).unwrap();
    write_geometries(&mut writer, traversals, &args.output_format);

    // dump the remaining nodes
    if args.untraversed {
        write_geometries(
            &mut writer,
            graph.node_weights().map(|p| Geometry::Point(*p)),
            &args.output_format,
        );
    }
}
