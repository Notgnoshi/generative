pub mod attractor;
#[cfg(feature = "cxx-bindings")]
mod cxxbridge;
pub mod dla;
pub mod flatten;
mod geometry_mut_map;
pub mod graph;
pub mod io;
#[cfg(feature = "cxx-bindings")]
pub mod noding;
pub mod snap;
pub mod triangulation;

pub use geometry_mut_map::MapCoordsInPlaceMut;

#[cfg(test)]
#[ctor::ctor]
fn init_test_logging() {
    let filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(tracing::Level::TRACE.into())
        .from_env_lossy();
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_ansi(true)
        .with_test_writer()
        .init();
}
