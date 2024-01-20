use crate::cxxbridge::GeometryCollectionShim;

#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("generative/generative/cxxbridge/coord_ffi.rs.h");
        type CoordShim = crate::cxxbridge::CoordShim;
        type LineStringShim = crate::cxxbridge::LineStringShim;
        type PolygonShim = crate::cxxbridge::PolygonShim;
    }

    extern "Rust" {
        type GeometryCollectionShim;

        fn get_total_geoms(self: &GeometryCollectionShim) -> usize;
        fn get_points(self: &GeometryCollectionShim) -> Vec<CoordShim>;
        fn get_linestrings(self: &GeometryCollectionShim) -> Vec<LineStringShim>;
        fn get_polygons(self: &GeometryCollectionShim) -> Vec<PolygonShim>;
    }
}
