#![feature(generic_associated_types)]

use kdtree::distance::squared_euclidean;
use kdtree::KdTree;
use std::io::Write;

mod cmdline;
mod wkio;
mod csiter;

use wkio::GatIterator; // :/

fn main() {
    let args = cmdline::Options::from_args();
    stderrlog::new()
        .module(module_path!())
        .quiet(args.quiet)
        .verbosity(args.verbose)
        .init()
        .unwrap();

    let mut writer = args.get_output_writer();
    let mut geoms = wkio::WktDeserializer::from_reader(args.get_input_reader());

    writeln!(&mut writer, "sample output").expect("Couldn't write?!");
    writer.flush().unwrap();

    // Scalar type, tag type, point type.
    let mut tree = KdTree::<f64, usize, [f64; 3]>::new(3);

    // An example.
    let point: [f64; 3] = [0.0, 0.1, 0.2];
    let data: usize = 0xdeadbeef;
    tree.add(point, data).unwrap();

    let query = [0.1, 0.1, 0.2];
    let _result = tree.within(&query, 0.1, &squared_euclidean).unwrap();

    // TODO: The geo::Geometry types are 2D only, so go back to using wkt::Geometry
    while let Some(_geom) = geoms.next() {
        // for prev, current in pairwise_points(geom) (implicitly convert to 3D, some edge cases
        // for closed geometries) TODO: How to handle GeometryCollections?
        //      lookup point in tree
        //      insert point in tree if it doesn't exist
        //      add point to graph
        //      mark current point as adjacent to previous.
    }
}
