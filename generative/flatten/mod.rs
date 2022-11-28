// TODO: Flatten Iterator<Item = Geometry<f64>> into an Iterator<Item = Point<f64>>
mod points;

// TODO: Flatten Iterator<Item = Geometry<f64>> into an Iterator<Item = Geometry<f64>> where
// MULTI-geometries get recursively flattened into their constituent geometries
//
// https://github.com/Notgnoshi/generative/issues/119
mod geometries;
