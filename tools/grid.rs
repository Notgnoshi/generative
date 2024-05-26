use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use generative::graph::GeometryGraph;
use generative::io::{
    get_output_writer, write_geometries, write_graph, GeometryFormat, GraphFormat,
};
#[cfg(feature = "cxx-bindings")]
use generative::noding::polygonize;
use geo::{Geometry, Point};
use petgraph::Undirected;
use stderrlog::ColorChoice;

#[derive(Debug, Clone, ValueEnum)]
enum GridFormat {
    /// Output the grid as a graph in TGF with WKT POINT node labels
    Graph,
    /// Output the grid lines in WKT
    Lines,
    /// Output the grid points in WKT
    Points,
    /// Output the grid cells as WKT POLYGONs
    #[cfg(feature = "cxx-bindings")]
    Cells,
}

impl std::fmt::Display for GridFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            // important: Should match clap::ValueEnum format
            GridFormat::Graph => write!(f, "graph"),
            GridFormat::Lines => write!(f, "lines"),
            GridFormat::Points => write!(f, "points"),
            #[cfg(feature = "cxx-bindings")]
            GridFormat::Cells => write!(f, "cells"),
        }
    }
}

#[derive(Debug, Clone, ValueEnum)]
enum GridType {
    Triangle,
    Quad,
    /// Quads, slanted to the right with ragged edges
    Ragged,
    Hexagon,
}
impl std::fmt::Display for GridType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            // important: Should match clap::ValueEnum format
            GridType::Triangle => write!(f, "triangle"),
            GridType::Quad => write!(f, "quad"),
            GridType::Ragged => write!(f, "ragged"),
            GridType::Hexagon => write!(f, "hexagon"),
        }
    }
}

/// Generate a regular grid graph
#[derive(Debug, Parser)]
#[clap(name = "grid", verbatim_doc_comment)]
struct CmdlineOptions {
    /// The log level
    #[clap(short, long, default_value_t = log::Level::Info)]
    log_level: log::Level,

    /// Output file to write result to. Defaults to stdout.
    #[clap(short, long)]
    output: Option<PathBuf>,

    /// Output geometry format.
    #[clap(short = 'O', long, default_value_t = GridFormat::Points)]
    output_format: GridFormat,

    /// The type of grid to generate
    #[clap(short, long, default_value_t = GridType::Quad)]
    grid_type: GridType,

    /// The number of cells along the x-axis
    #[clap(short = 'W', long, default_value_t = 5)]
    width: usize,

    /// The number of cells along the y-axis
    #[clap(short = 'H', long, default_value_t = 5)]
    height: usize,

    /// The size of each grid cell. Use --size-x or --size-y to specify the size for the different axes
    #[clap(short = 's', long)]
    size: Option<f64>,

    /// The width of each grid cell
    #[clap(long)]
    size_x: Option<f64>,

    /// The height of each grid cell
    #[clap(long)]
    size_y: Option<f64>,
}

fn grid(
    width: usize,
    height: usize,
    size_x: f64,
    size_y: f64,
    grid_type: GridType,
) -> GeometryGraph<Undirected> {
    if width == 0 || height == 0 {
        return GeometryGraph::<Undirected>::default();
    }

    match grid_type {
        GridType::Triangle => tri_grid(width, height, size_x, size_y),
        GridType::Quad => quad_grid(width, height, size_x, size_y),
        GridType::Ragged => ragged_grid(width, height, size_x, size_y),
        GridType::Hexagon => hex_grid(width, height, size_x, size_y),
    }
}

fn tri_i2x(i: usize, size_x: f64, odd: bool) -> f64 {
    let mut x = (i as f64) * size_x;
    if odd {
        x += 0.5 * size_x;
    }
    x
}

fn tri_j2y(j: usize, triangle_height: f64) -> f64 {
    (j as f64) * triangle_height
}

fn tri_grid(width: usize, height: usize, size_x: f64, size_y: f64) -> GeometryGraph<Undirected> {
    let nodes = (width + 1) * (height + 1);
    let edges = 2 * nodes;
    let mut graph = GeometryGraph::<Undirected>::with_capacity(nodes, edges);

    let triangle_height = f64::sqrt(size_y.powi(2) - (size_x.powi(2) / 4.0));

    // Add the nodes
    for j in 0..=height {
        for i in 0..=width {
            let node_index = (width + 1) * j + i;
            let odd_row = j % 2 != 0;
            let x = tri_i2x(i, size_x, odd_row);
            let y = tri_j2y(j, triangle_height);
            let point = Point::new(x, y);
            let index = graph.add_node(point);
            log::trace!("id={} node={point:?}", index.index());
            debug_assert_eq!(index.index(), node_index);
        }
    }

    // Add the edges
    for j in 0..=height {
        for i in 0..=width {
            let current_index = (width + 1) * j + i;
            let odd_row = j % 2 != 0;
            log::trace!("adding neighbors for id={current_index}");

            // Start to the left of the current node, and work clockwise around its neighbors.
            //
            // Around the left and right border, only the odd rows have upper and lower left
            // neighbors, and only the even rows have upper and lower right neighbors.
            //
            // even j=0  0---1---2
            //            \ / \ / \
            // odd  j=1    3---4---5
            //            / \ / \ /
            // even j=2  6---7---8
            if i > 0 {
                let left = current_index - 1;
                log::trace!("added left id={left}");
                graph.update_edge(current_index.into(), left.into(), ());
            }
            if j > 0 && (i > 0 || odd_row) {
                let upper_left = if odd_row {
                    current_index - (width + 1)
                } else {
                    current_index - (width + 2)
                };
                log::trace!("added upper left id={upper_left}");
                graph.update_edge(current_index.into(), upper_left.into(), ());
            }
            if j > 0 && (!odd_row || i < width) {
                let upper_right = if odd_row {
                    current_index - width
                } else {
                    current_index - (width + 1)
                };
                log::trace!("added upper right id={upper_right}");
                graph.update_edge(current_index.into(), upper_right.into(), ());
            }
            if i < width {
                let right = current_index + 1;
                log::trace!("added right id={right}");
                graph.update_edge(current_index.into(), right.into(), ());
            }
            if j < height && (!odd_row || i < width) {
                let lower_right = if odd_row {
                    current_index + width + 2
                } else {
                    current_index + width + 1
                };
                log::trace!("added lower right id={lower_right}");
                graph.update_edge(current_index.into(), lower_right.into(), ());
            }
            if j < height && (i > 0 || odd_row) {
                let lower_left = if odd_row {
                    current_index + width + 1
                } else {
                    current_index + width
                };
                log::trace!("added lower left id={lower_left}");
                graph.update_edge(current_index.into(), lower_left.into(), ());
            }
        }
    }

    graph
}

fn quad_i2x(i: usize, delta_x: f64, min_x: f64) -> f64 {
    (i as f64) * delta_x + min_x
}

fn quad_j2y(j: usize, delta_y: f64, min_y: f64) -> f64 {
    (j as f64) * delta_y + min_y
}

fn quad_grid(width: usize, height: usize, size_x: f64, size_y: f64) -> GeometryGraph<Undirected> {
    let nodes = (width + 1) * (height + 1);
    let edges = 2 * width * height - width - height;
    let mut graph = GeometryGraph::<Undirected>::with_capacity(nodes, edges);

    // Add the nodes
    for j in 0..=height {
        for i in 0..=width {
            let x = quad_i2x(i, size_x, 0.0);
            let y = quad_j2y(j, size_y, 0.0);
            let point = Point::new(x, y);
            let index = graph.add_node(point);
            let node_index = (width + 1) * j + i;
            log::trace!("id={} node={point:?}", index.index());
            debug_assert_eq!(index.index(), node_index);
        }
    }

    // Add the edges
    for j in 0..=height {
        for i in 0..=width {
            // As an implementation detail, the GeometryGraph gives nodes integer IDs in the order
            // they were added. This is the index of the current node in the graph.
            let current_index = (width + 1) * j + i;
            log::trace!("adding neighbors for id={current_index}");

            // Add the four neighbors, starting at the left and working clockwise
            //
            // 0--1--2
            // |  |  |
            // 3--4--5
            // |  |  |
            // 6--7--8
            if i > 0 {
                let left = current_index - 1;
                log::trace!("added left id={left}");
                graph.update_edge(current_index.into(), left.into(), ());
            }
            if j < height {
                let upper = current_index + width + 1;
                log::trace!("added upper id={upper}");
                graph.update_edge(current_index.into(), upper.into(), ());
            }
            if i < width {
                let right = current_index + 1;
                log::trace!("added right id={right}");
                graph.update_edge(current_index.into(), right.into(), ());
            }
            if j > 0 {
                let lower = current_index - width - 1;
                log::trace!("added lower id={lower}");
                graph.update_edge(current_index.into(), lower.into(), ());
            }
        }
    }
    graph
}

fn ragged_grid(width: usize, height: usize, size_x: f64, size_y: f64) -> GeometryGraph<Undirected> {
    let nodes = (width + 1) * (height + 1);
    let edges = 2 * width * height - width - height;
    let mut graph = GeometryGraph::<Undirected>::with_capacity(nodes, edges);

    // Add the nodes
    for j in 0..=height {
        for i in 0..=(width + 1) {
            let x = quad_i2x(i, size_x, 0.0);
            let y = quad_j2y(j, size_y, 0.0);
            let point = Point::new(x, y);
            let index = graph.add_node(point);
            let node_index = (width + 2) * j + i;
            log::trace!("id={} node={point:?}", index.index());
            debug_assert_eq!(index.index(), node_index);
        }
    }

    // Add the edges
    for j in 0..=height {
        // Because of the ragged edges, you need one more index to get another column of cells
        for i in 0..=(width + 1) {
            let current_index = (width + 2) * j + i;
            log::trace!("adding neighbors for id={current_index}");

            // Add the four neighbors, starting at the left and working clockwise
            //
            // 0-1-2-3
            //  / / /
            // 4-5-6-7
            //  / / /
            // 8-9-0-1
            if i > 0 {
                let left = current_index - 1;
                log::trace!("added left id={left}");
                graph.update_edge(current_index.into(), left.into(), ());
            }
            if j > 0 {
                let upper = current_index - (width + 1);
                log::trace!("added upper id={upper}");
                graph.update_edge(current_index.into(), upper.into(), ());
            }
            if i < (width + 1) {
                let right = current_index + 1;
                log::trace!("added right id={right}");
                graph.update_edge(current_index.into(), right.into(), ());
            }
            if j < height {
                let lower = current_index + width + 1;
                log::trace!("added lower id={lower}");
                graph.update_edge(current_index.into(), lower.into(), ());
            }
        }
    }
    graph
}

fn hex_grid(width: usize, height: usize, size_x: f64, _size_y: f64) -> GeometryGraph<Undirected> {
    // Ignore size_y, because this is limited to regular hexagons.
    let outradius = size_x;
    let inradius = f64::sqrt(3.0) * outradius / 2.0;

    // Adding the nodes hexagon-by-hexagon results in a bad time (it's difficult to construct an
    // indexing scheme that makes sense for adding the edges; additionally, accumulating floating
    // point errors result in "duplicate" nodes being added that aren't bitwise equal).
    //
    // So throw out the hexagon geometry alltogether and take a topological approach. That will
    // give us an indexing scheme that's easier to work with.
    //
    // even       0---1   +   2---3   +   4                         0--1  2--3  4
    //           /     \     /     \                                |  |  |  |
    // odd      5   +   6---7   +   8---9                           5  6--7  8--9
    //           \     /     \     /     \                          |  |  |  |  |
    // even       0---1   +   2---3   +   4                         0--1  2--3  4
    //           /     \     /     \     /       ===topology===>    |  |  |  |  |
    // odd      5   +   6---7   +   8---9                           5  6--7  8--9
    //           \     /     \     /     \                          |  |  |  |  |
    // even       0---1   +   2---3   +   4                         0--1  2--3  4
    //                 \     /     \     /                             |  |  |  |
    // odd      5   +   6---7   +   8---9                           5  6--7  8--9
    //
    // Notice that this scheme gives two extra nodes that will need to be removed after adding all
    // the edges (because removing them will re-adjust all the node indices).
    let two_extras = 2;
    let nodes = height * (2 * width + 2) + 2 * width + two_extras;
    let edges = height * (3 * width + 2) + 2 * width;
    let mut graph = GeometryGraph::<Undirected>::with_capacity(nodes, edges);

    let rows = 2 * height + 2;
    let cols = width + 1;

    let wide_offset = 2.0 * outradius;
    let narrow_offset = outradius;

    for row in 0..rows {
        let odd_row = row % 2 != 0;
        let mut base_x = 0.0;
        if odd_row {
            base_x -= 0.5 * outradius;
        }
        let base_y = (row as f64) * inradius;

        let mut x_offset = 0.0;
        for col in 0..cols {
            let even_col = col % 2 == 0;
            if col == 0 {
                // no offset on the base
            } else if (odd_row && even_col) || (!odd_row && !even_col) {
                x_offset += narrow_offset;
            } else {
                x_offset += wide_offset;
            }

            let x = base_x + x_offset;
            let point = Point::new(x, base_y);
            let index = graph.add_node(point);
            log::trace!("id={} node={point:?}", index.index());
        }
    }

    let cols_is_even = width % 2 == 0;
    let adjacency_offset = width + 1;
    let mut n = 0;
    for row in 0..rows {
        let even_row = row % 2 == 0;
        for _col in 0..cols {
            log::trace!("adding neighbors for id={n}");
            if n > adjacency_offset {
                let upper = n - adjacency_offset;
                log::trace!("added upper id={upper}");
                graph.update_edge(n.into(), upper.into(), ());
            }
            if n < (nodes - adjacency_offset) {
                let lower = n + adjacency_offset;
                log::trace!("added lower id={lower}");
                graph.update_edge(n.into(), lower.into(), ());
            }

            // Holy shit there are so many god damn edge cases. This is ridiculous.
            let is_furthest_right = (n + 1) % adjacency_offset == 0;
            let even_index = n % 2 == 0;
            let has_right_neighbor = if cols_is_even || even_row {
                even_index && !is_furthest_right
            } else {
                !even_index && !is_furthest_right
            };

            if has_right_neighbor {
                let right = n + 1;
                log::trace!("added right id={right}");
                graph.update_edge(n.into(), right.into(), ());
            }

            n += 1;
        }
    }

    // Remove the two dangling extra nodes that were added to make the indexing math suck
    // (slightly) less.
    let nodes_to_remove = if cols_is_even {
        [width, nodes - cols]
    } else {
        [nodes - cols, nodes - 1]
    };
    graph.retain_nodes(move |_, idx| !nodes_to_remove.contains(&idx.index()));

    graph
}

fn main() {
    let args = CmdlineOptions::parse();

    stderrlog::new()
        .verbosity(args.log_level)
        .color(ColorChoice::Auto)
        .init()
        .expect("Failed to initialize stderrlog");

    let (mut size_x, mut size_y) = if let Some(size) = args.size {
        (size, size)
    } else {
        (1.0, 1.0)
    };
    if let Some(size) = args.size_x {
        size_x = size;
    }
    if let Some(size) = args.size_y {
        size_y = size;
    }

    let graph = grid(args.width, args.height, size_x, size_y, args.grid_type);

    let writer = get_output_writer(&args.output).unwrap();
    match args.output_format {
        GridFormat::Graph => write_graph(writer, &graph, &GraphFormat::Tgf),
        GridFormat::Lines => write_graph(writer, &graph, &GraphFormat::Wkt),
        GridFormat::Points => write_geometries(
            writer,
            graph.node_weights().map(|p| Geometry::Point(*p)),
            GeometryFormat::Wkt,
        ),
        #[cfg(feature = "cxx-bindings")]
        GridFormat::Cells => {
            let (polygons, dangles) = polygonize(&graph);
            let polygons = polygons.into_iter().map(Geometry::Polygon);
            let dangles = dangles.into_iter().map(Geometry::LineString);
            let geoms = polygons.chain(dangles);
            write_geometries(writer, geoms, GeometryFormat::Wkt);
        }
    }
}
