#[cfg(feature = "geom2graph-bindings")]
mod cxxbridge;
pub mod dla;
pub mod flatten;
mod geometry_mut_map;
pub mod graph;
pub mod io;
#[cfg(feature = "geom2graph-bindings")]
pub mod noding;
pub mod snap;
pub mod triangulation;

pub use geometry_mut_map::MapCoordsInPlaceMut;

#[cfg(test)]
#[ctor::ctor]
fn init_test_logging() {
    stderrlog::new()
        .verbosity(log::Level::Trace)
        .color(stderrlog::ColorChoice::Auto)
        .init()
        .expect("Failed to initialize stderrlog");
}
