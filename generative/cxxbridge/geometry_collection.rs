use crate::cxxbridge::{CoordShim, LineStringShim, PolygonShim};
use crate::flatten::flatten_nested_geometries;

/// A wrapper around a [geo::GeometryCollection] to enable passing it across an FFI boundary
pub struct GeometryCollectionShim {
    geoms: Vec<geo::Geometry>,
}

impl GeometryCollectionShim {
    pub fn new<G>(geoms: G) -> Self
    where
        G: IntoIterator<Item = geo::Geometry>,
    {
        Self {
            geoms: flatten_nested_geometries(geoms).collect(),
        }
    }

    pub fn get_total_geoms(&self) -> usize {
        self.geoms.len()
    }
    pub fn get_geo_points(&self) -> Vec<geo::Point> {
        let mut points = Vec::new();
        let mut num_points = 0;
        for g in &self.geoms {
            match g {
                geo::Geometry::Point(_) => num_points += 1,
                geo::Geometry::MultiPoint(m) => num_points += m.len(),
                _ => {}
            }
        }
        points.reserve_exact(num_points);

        for g in self.geoms.iter() {
            match g {
                geo::Geometry::Point(p) => {
                    points.push(*p);
                }
                geo::Geometry::MultiPoint(m) => {
                    for p in m.0.iter() {
                        points.push(*p);
                    }
                }
                _ => {}
            }
        }
        points
    }

    pub fn get_points(&self) -> Vec<CoordShim> {
        let mut points = Vec::new();
        let mut num_points = 0;
        for g in &self.geoms {
            match g {
                geo::Geometry::Point(_) => num_points += 1,
                geo::Geometry::MultiPoint(m) => num_points += m.len(),
                _ => {}
            }
        }
        points.reserve_exact(num_points);

        for g in self.geoms.iter() {
            match g {
                geo::Geometry::Point(p) => {
                    points.push(CoordShim { x: p.0.x, y: p.0.y });
                }
                geo::Geometry::MultiPoint(m) => {
                    for p in m.0.iter() {
                        points.push(CoordShim { x: p.0.x, y: p.0.y });
                    }
                }
                _ => {}
            }
        }

        points
    }

    fn from_linestring(l: &geo::LineString) -> LineStringShim {
        LineStringShim {
            vec: l.0.iter().map(|c| (*c).into()).collect(),
        }
    }

    pub fn get_linestrings(&self) -> Vec<LineStringShim> {
        let mut linestrings = Vec::<LineStringShim>::new();
        for g in &self.geoms {
            match g {
                geo::Geometry::Line(l) => {
                    let line = vec![l.start.into(), l.end.into()];
                    linestrings.push(LineStringShim { vec: line });
                }
                geo::Geometry::LineString(l) => {
                    linestrings.push(Self::from_linestring(l));
                }
                geo::Geometry::MultiLineString(m) => {
                    for l in &m.0 {
                        linestrings.push(Self::from_linestring(l));
                    }
                }
                _ => {}
            }
        }

        linestrings
    }

    fn from_poly(p: &geo::Polygon) -> PolygonShim {
        let mut rings = Vec::new();
        rings.push(Self::from_linestring(p.exterior()));
        for hole in p.interiors() {
            rings.push(Self::from_linestring(hole));
        }
        PolygonShim { vec: rings }
    }

    pub fn get_polygons(&self) -> Vec<PolygonShim> {
        // Each polygon is a Vec<Vec<CoordShim>>, where the first vec is the exterior shells, and
        // any following are interior holes
        let mut polygons = Vec::new();
        for g in &self.geoms {
            match g {
                geo::Geometry::Polygon(p) => polygons.push(Self::from_poly(p)),
                geo::Geometry::MultiPolygon(m) => {
                    for p in &m.0 {
                        polygons.push(Self::from_poly(p));
                    }
                }
                geo::Geometry::Rect(r) => polygons.push(Self::from_poly(&r.to_polygon())),
                geo::Geometry::Triangle(t) => polygons.push(Self::from_poly(&t.to_polygon())),
                _ => {}
            }
        }

        polygons
    }
}

unsafe impl cxx::ExternType for GeometryCollectionShim {
    type Id = cxx::type_id!("GeometryCollectionShim");
    type Kind = cxx::kind::Opaque;
}
