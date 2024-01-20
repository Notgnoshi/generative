#[cxx::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("geometry_graph.hpp");

        type GeometryGraphShim;
        type CoordShim = crate::cxxbridge::CoordShim;
        type GraphEdge = crate::cxxbridge::GraphEdge;

        fn nodes(self: &GeometryGraphShim) -> Vec<CoordShim>;
        fn edges(self: &GeometryGraphShim) -> Vec<GraphEdge>;
    }

    impl UniquePtr<GeometryGraphShim> {}
}
