# Generative / computational art in Rust

This is a Rust library for generative art that is wrapped in a [PyO3](https://github.com/PyO3/pyo3) wrapper to expose the library to the top level Python library and tools.

## TODO

The DLA model has the following public interface

```rust
pub struct Particle {
    pub coordinates: [f64; 2],
    pub join_attempts: usize,
}

pub type GraphType = Graph<Particle, (), Undirected>;

pub struct Model {
    pub particle_graph: GraphType,
}

impl Model {
    pub fn new(
        dimensions: u8,
        seeds: usize,
        seed: u64,
        particle_spacing: f64,
        attraction_distance: f64,
        min_move_dstance: f64,
        stubbornness: usize,
        stickiness: f64,
    ) -> Model;

    pub fn run(&mut self, particles: usize);
}
```

This could have been implemented as

```rust
pub struct DLAParameters {
    pub dimensions: u8,
    pub seeds: usize,
    pub seed: u64,
    pub particle_spacing: f64,
    pub attraction_distance: f64,
    pub min_move_dstance: f64,
    pub stubbornness: usize,
    pub stickiness: f64,
}

pub struct Particle {
    pub coordinates: [f64; 2],
    pub join_attempts: usize,
}

pub type GraphType = Graph<Particle, (), Undirected>;

pub fn dla(
    parameters: DLAParameters,
    particles: usize,
) -> GraphType;
```

The downside is that you can't run it for a bit, stop and work with it, and then run it a little bit more.
I think I like the original Model-based interface a bit more.
It also allows for seeding the model with an initial graph of particles.

The DLA model should be refactored into

```rust
pub struct Particle {
    // TODO: Allow 3D graphs
    pub coordinates: [f64; 2],
    pub join_attempts: usize,
}
// TODO: Is there a way to alias Graph so that I can name this type Graph?
pub type DLAGraph = Graph<Particle, (), Undirected>;

pub struct Parameters {
    pub seeds: usize,
    pub seed: u64,
    pub particle_spacing: f64,
    pub attraction_distance: f64,
    pub min_move_dstance: f64,
    pub stubbornness: usize,
    pub stickiness: f64,
}

pub struct Model {
    pub particle_graph: DLAGraph,
}

// TODO: As a Rust newbie, I don't really know what should go in the trait, and what shouldn't.
// TODO: Make the dimensionality generic
trait ModelInterface {
    fn new(params: Parameters) -> Model;
    fn run(&mut self, particles: usize);

    fn nearest_particle(&self, coords: &[f64; 2]) -> Option<NodeIndex>;

    fn add_seeds(&mut self, num_seeds: usize);
    fn add_random_seed(&mut self) -> NodeIndex;
    fn add_seed(&mut self, coords: &[f64; 2]) -> NodeIndex;

    fn add_particles(&mut self, num_particles: usize);
    fn add_random_particle(&mut self) -> NodeIndex;
    fn add_particle(&mut self, parent_index: NodeIndex, coords: &[f64; 2]) -> NodeIndex;
}
```

In addition to this interface, the `petgraph::Graph` needs to be wrapped.
Care should be taken to convert it to a graph type that works well enough with the C++ `geom2graph` graph type.
