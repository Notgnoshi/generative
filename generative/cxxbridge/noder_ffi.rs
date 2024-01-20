#[cxx::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("noder.hpp");

        type GeometryCollectionShim = crate::cxxbridge::GeometryCollectionShim;
        type GeometryGraphShim = crate::cxxbridge::GeometryGraphShim;

        /// Node the given collection of geometries
        ///
        /// # Safety
        ///
        /// The noding is done by either a geos SnappingNoder or IteratedNoder, both of which have
        /// some gotchas.
        ///
        /// * The IteratedNoder can throw topology exceptions if it doesn't converge by MAX_ITERS
        /// * The SnappingNoder doesn't handle isolated POINTs
        unsafe fn node(
            geoms: &GeometryCollectionShim,
            tolerance: f64,
        ) -> UniquePtr<GeometryGraphShim>;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cxxbridge::{CoordShim, GraphEdge};

    #[test]
    fn test_noder_doesnt_crash() {
        let empty = crate::cxxbridge::GeometryCollectionShim::new(Vec::new());
        let tolerance = 0.001;
        let graph = unsafe { ffi::node(&empty, tolerance) };

        assert!(!graph.is_null());
    }

    #[test]
    fn test_noder_isolated_points() {
        let geoms = [geo::Point::new(0.0, 0.0), geo::Point::new(0.0, 1.0)];
        let geoms: Vec<_> = geoms.into_iter().map(geo::Geometry::Point).collect();
        let geoms = crate::cxxbridge::GeometryCollectionShim::new(geoms);

        let tolerance = 0.0;
        let graph = unsafe { ffi::node(&geoms, tolerance) };
        assert!(!graph.is_null());

        let nodes = graph.nodes();
        let expected = [CoordShim { x: 0.0, y: 0.0 }, CoordShim { x: 0.0, y: 1.0 }];
        assert_eq!(nodes, expected);

        let edges = graph.edges();
        assert!(edges.is_empty());
    }

    #[test]
    fn test_noder_linestring() {
        let line: geo::LineString = vec![(0.0, 0.0), (0.00001, 0.0), (2.0, 0.0)].into();
        let geoms = vec![geo::Geometry::LineString(line)];
        let geoms = crate::cxxbridge::GeometryCollectionShim::new(geoms);

        let tolerance = 0.0001;
        let graph = unsafe { ffi::node(&geoms, tolerance) };
        assert!(!graph.is_null());

        let nodes = graph.nodes();
        let expected = [CoordShim { x: 0.0, y: 0.0 }, CoordShim { x: 2.0, y: 0.0 }];
        assert_eq!(nodes, expected);

        let edges = graph.edges();
        let expected = [GraphEdge { src: 0, dst: 1 }];
        assert_eq!(edges, expected);
    }
}
