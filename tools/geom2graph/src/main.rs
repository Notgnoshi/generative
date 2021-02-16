use geos::Geom;
use kdtree::distance::squared_euclidean;
use kdtree::KdTree;
use log::trace;
use std::io::Write;

mod cmdline;
mod csiter;
mod deepflatten;
mod geomflattener;
mod wkio;

use crate::deepflatten::DeepFlattenExt;

fn main() {
    let args = cmdline::Options::from_args();
    stderrlog::new()
        .module(module_path!())
        .quiet(args.quiet)
        .verbosity(args.verbose)
        .init()
        .unwrap();

    let mut writer = args.get_output_writer();
    let geometries = wkio::WktDeserializer::from_reader(args.get_input_reader());
    let geometries = geometries.deep_flatten();

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

    for geometry in geometries {
        let geometries = geomflattener::GeometryIterator::new(&geometry);
        for geometry in geometries {
            // TODO: Ignore multi-geometries because we can't create a PointIterator from them.
            let points = csiter::PointIterator::new_from_const_geom(geometry);
            for point in points {
                let wkt = point.to_wkt().unwrap();
                trace!("point: {}", wkt);
                // for prev, current in pairwise_points(geom) (implicitly convert to 3D, some edge cases
                // for closed geometries) TODO: How to handle GeometryCollections?
                //      lookup point in tree
                //      insert point in tree if it doesn't exist
                //      add point to graph
                //      mark current point as adjacent to previous.
            }
        }
    }
}
