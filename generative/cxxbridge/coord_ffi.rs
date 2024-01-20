#[cxx::bridge]
pub mod ffi {
    #[derive(Debug, Clone, Copy, PartialEq)]
    struct CoordShim {
        x: f64,
        y: f64,
    }

    impl Vec<CoordShim> {}

    #[derive(Debug, Clone, PartialEq)]
    struct LineStringShim {
        vec: Vec<CoordShim>,
    }
    impl Vec<LineStringShim> {}

    #[derive(Debug, Clone, PartialEq)]
    struct PolygonShim {
        vec: Vec<LineStringShim>,
    }
    impl Vec<PolygonShim> {}

    #[derive(Debug, Clone, Copy, PartialEq)]
    struct GraphEdge {
        src: usize,
        dst: usize,
    }
    impl Vec<GraphEdge> {}
}

impl From<geo::Coord> for ffi::CoordShim {
    fn from(c: geo::Coord) -> ffi::CoordShim {
        ffi::CoordShim { x: c.x, y: c.y }
    }
}
