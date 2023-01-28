mod stdio;
mod tgf;
mod wkt;

pub use stdio::{get_input_reader, get_output_writer};
// TODO: A read_graph method that takes in the GraphFormat (the same as read_geometries) would
// require FFI bindings to geom2graph. See: https://github.com/Notgnoshi/generative/issues/130
pub use tgf::{read_tgf_graph, write_graph, GraphFormat};

pub use self::wkt::{
    read_geometries, read_wkt_geometries, read_wkt_geometries_and_styles, write_geometries,
    write_wkt_geometries, GeometryAndStyle, GeometryFormat, SvgStyle,
};
