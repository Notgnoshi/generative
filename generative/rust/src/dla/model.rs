use kdtree::distance::squared_euclidean;
use kdtree::KdTree;
use log::{debug, info, trace, warn};
use petgraph::graph::{Graph, NodeIndex};
use petgraph::Undirected;
use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;

type DimensionType = f64;

#[derive(Debug, Clone, Copy)]
pub struct Particle {
    /// For best results, you probably want 2 or 3 dimensions.
    /// TODO: Variable dimensionality
    /// TODO: This is best if it's an ndarray::Array1
    pub coordinates: [f64; 2],
    /// The number of times this particle has been attempted to be joined to.
    pub join_attempts: usize,
}

/// Data associated with the nodes.
type NodeDataType = Particle;
/// Data associated with the edges.
type EdgeDataType = ();
pub type GraphType = Graph<NodeDataType, EdgeDataType, Undirected>;

#[derive(Debug)]
pub struct Parameters {
    pub seeds: usize,
    pub seed: u64,
    pub particle_spacing: f64,
    pub attraction_distance: f64,
    pub min_move_distance: f64,
    pub stubbornness: usize,
    pub stickiness: f64,
}

#[derive(Debug)]
pub struct Model {
    /// The particles and their parent associations.
    pub particle_graph: GraphType,

    /// The particle graph holds all of the Particles. The spatial index just exists
    /// to accelerate nearest neighbor lookup, and will hold the coordinates and
    /// node indices into the particle graph.
    /// TODO: Variable dimensionality
    /// TODO: Figure out how to store a reference to an ndarray::Array1 in the index
    index: KdTree<DimensionType, NodeIndex, [DimensionType; 2]>,
    rng: StdRng,

    /// Particle dimensionality
    dimensions: u8,

    // Tunable parameters
    bounding_radius: f64,
    particle_spacing: f64,
    attraction_distance: f64,
    min_move_distance: f64,
    stubbornness: usize,
    stickiness: f64,
}

impl Model {
    /// Create a new model with the given tunable parameters.
    pub fn new(params: Parameters) -> Model {
        let seed = Model::generate_random_seed_if_not_specified(params.seed);
        info!("Intializing rng with seed {}", seed);

        debug!("Initializing model with parameters {:?}", params);

        let mut model = Model {
            particle_graph: Graph::new_undirected(),
            // TODO: Variable dimensionality
            index: KdTree::new(2),
            rng: StdRng::seed_from_u64(seed),
            dimensions: 2,
            bounding_radius: 5.0,
            particle_spacing: params.particle_spacing,
            attraction_distance: params.attraction_distance,
            min_move_distance: params.min_move_distance,
            stubbornness: params.stubbornness,
            stickiness: params.stickiness,
        };

        if params.seeds == 0 {
            warn!("Cannot run DLA model with no initial seed particles. Using one seed.");
        }
        model.add_seeds(if params.seeds == 0 { 1 } else { params.seeds });

        return model;
    }

    /// Add the specified number of particles to the model.
    pub fn run(&mut self, particles: usize) {
        debug!("Adding {} particles", particles);
        for _ in 0..particles {
            self.add_particle();
        }
    }

    /// Add a single particle to the DLA model.
    fn add_particle(&mut self) {
        let mut coords = self.generate_random_coord();

        loop {
            let distance: f64;
            let nearest_index: NodeIndex;

            {
                let nearest = self
                    .index
                    .nearest(&coords, 1, &squared_euclidean)
                    .expect("Failed to find nearest particles");

                let results = *nearest.first().expect("Failed to get nearest particle");
                distance = results.0;
                nearest_index = *results.1;
            }

            if distance < self.attraction_distance {
                if self.attempt_to_join(&mut coords, nearest_index) {
                    trace!("Added particle POINT({} {})", coords[0], coords[1]);
                    return;
                }
            } else {
                // Random walk
                let v = &coords;
                let m = f64::max(self.min_move_distance, distance - self.attraction_distance);
                let u = Model::norm(&[self.rng.gen_range(0.0..1.0), self.rng.gen_range(0.0..1.0)]);
                coords = [v[0] + u[0] * m, v[1] + u[1] * m];

                if Model::length(&coords) > self.bounding_radius * 2.0 {
                    coords = self.generate_random_coord();
                }
            }
        }
    }

    /// Add starting seeds to the DLA model.
    fn add_seeds(&mut self, particles: usize) {
        debug!("Adding {} seed particles", particles);

        for _ in 0..particles {
            let coords = self.generate_random_coord();
            let particle = Particle {
                // TODO: Variable dimensionality
                coordinates: if particles == 1 {
                    [0.0, 0.0]
                } else {
                    // TODO: Maybe there's other seed patterns that could be neat.
                    [
                        coords[0] * (5.0 + particles as f64 / 10.0),
                        coords[1] * (5.0 + particles as f64 / 10.0),
                    ]
                },
                join_attempts: 0,
            };
            let particle_index = self.particle_graph.add_node(particle);
            // TODO: How to avoid copying the coordinates?
            self.index
                .add(particle.coordinates, particle_index)
                .expect("Failed to add seed to spatial index");
        }
    }

    fn attempt_to_join(&mut self, new_coords: &mut [f64; 2], parent_index: NodeIndex) -> bool {
        // Get parent particle from handle on parent.
        let mut parent = self.particle_graph.node_weight_mut(parent_index).expect(
            format!(
                "Failed to get node from parent index {}",
                parent_index.index()
            )
            .as_str(),
        );
        parent.join_attempts += 1;

        if parent.join_attempts >= self.stubbornness {
            if self.rng.gen_range(0.0..1.0) <= self.stickiness {
                // Bump the new particle away from the parent by the particle spacing
                *new_coords = Model::lerp(&parent.coordinates, &new_coords, self.particle_spacing);
                self.bounding_radius = self
                    .bounding_radius
                    .max(Model::length(new_coords) + self.attraction_distance);

                let new_particle = Particle {
                    coordinates: *new_coords,
                    join_attempts: 0,
                };

                // Place the particle in the graph and the spatial index
                let graph_index = self.particle_graph.add_node(new_particle);
                self.particle_graph.add_edge(graph_index, parent_index, ());
                self.index
                    .add(new_particle.coordinates, graph_index)
                    .expect("Failed to add new particle to index");

                return true;
            }
        }

        // Nudge the new particle
        *new_coords = Model::lerp(
            &parent.coordinates,
            &new_coords,
            self.attraction_distance + self.min_move_distance,
        );

        return false;
    }

    fn generate_random(&mut self) -> f64 {
        self.rng
            .gen_range(-self.bounding_radius..self.bounding_radius)
    }

    // TODO: ndarray::Array1
    fn generate_random_coord(&mut self) -> [f64; 2] {
        [self.generate_random(), self.generate_random()]
    }

    fn length(v: &[f64; 2]) -> f64 {
        (v[0] * v[0] + v[1] * v[1]).sqrt()
    }
    fn norm(v: &[f64; 2]) -> [f64; 2] {
        let l = Model::length(v);
        [v[0] / l, v[1] / l]
    }

    fn lerp(a: &[f64; 2], b: &[f64; 2], d: f64) -> [f64; 2] {
        // a + unit(b - a) * d
        let u = [b[0] - a[0], b[1] - a[1]];
        let u = Model::norm(&u);
        [a[0] + u[0] * d, a[1] + u[1] * d]
    }

    /// Generate a random seed, or pass one through if specified.
    fn generate_random_seed_if_not_specified(seed: u64) -> u64 {
        if seed == 0 {
            let mut rng = rand::thread_rng();
            rng.gen()
        } else {
            seed
        }
    }
}
