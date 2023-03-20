use geo::{
    Coord, CoordNum, Geometry, GeometryCollection, Line, LineString, MultiLineString, MultiPoint,
    MultiPolygon, Point, Polygon, Rect, Triangle,
};

pub trait MapCoordsInPlaceMut<T> {
    fn map_coords_in_place_mut(&mut self, func: impl FnMut(Coord<T>) -> Coord<T>)
    where
        T: CoordNum;
}

impl<T: CoordNum> MapCoordsInPlaceMut<T> for Point<T> {
    fn map_coords_in_place_mut(&mut self, mut func: impl FnMut(Coord<T>) -> Coord<T>) {
        self.0 = func(self.0);
    }
}
impl<T: CoordNum> MapCoordsInPlaceMut<T> for Line<T> {
    fn map_coords_in_place_mut(&mut self, mut func: impl FnMut(Coord<T>) -> Coord<T>) {
        self.start = func(self.start);
        self.end = func(self.end);
    }
}
impl<T: CoordNum> MapCoordsInPlaceMut<T> for LineString<T> {
    fn map_coords_in_place_mut(&mut self, mut func: impl FnMut(Coord<T>) -> Coord<T>) {
        for p in &mut self.0 {
            *p = func(*p);
        }
    }
}
impl<T: CoordNum> MapCoordsInPlaceMut<T> for Polygon<T> {
    fn map_coords_in_place_mut(&mut self, mut func: impl FnMut(Coord<T>) -> Coord<T>) {
        // API design is hard. There's no way to use a FnMut on a Polygon without cloning the
        // Polygon :(
        let poly = self.clone();
        let (mut ext, mut ints) = poly.into_inner();
        ext.map_coords_in_place_mut(&mut func);
        for int in &mut ints {
            int.map_coords_in_place_mut(&mut func);
        }
        let mut poly = Polygon::new(ext, ints);
        std::mem::swap(self, &mut poly);
    }
}
impl<T: CoordNum> MapCoordsInPlaceMut<T> for Rect<T> {
    fn map_coords_in_place_mut(&mut self, mut func: impl FnMut(Coord<T>) -> Coord<T>) {
        let mut rect = Rect::new(func(self.min()), func(self.max()));
        std::mem::swap(self, &mut rect);
    }
}
impl<T: CoordNum> MapCoordsInPlaceMut<T> for Triangle<T> {
    fn map_coords_in_place_mut(&mut self, mut func: impl FnMut(Coord<T>) -> Coord<T>) {
        self.0 = func(self.0);
        self.1 = func(self.1);
        self.2 = func(self.2);
    }
}
impl<T: CoordNum> MapCoordsInPlaceMut<T> for MultiPoint<T> {
    fn map_coords_in_place_mut(&mut self, mut func: impl FnMut(Coord<T>) -> Coord<T>) {
        for p in &mut self.0 {
            p.map_coords_in_place_mut(&mut func);
        }
    }
}
impl<T: CoordNum> MapCoordsInPlaceMut<T> for MultiLineString<T> {
    fn map_coords_in_place_mut(&mut self, mut func: impl FnMut(Coord<T>) -> Coord<T>) {
        for l in &mut self.0 {
            l.map_coords_in_place_mut(&mut func);
        }
    }
}
impl<T: CoordNum> MapCoordsInPlaceMut<T> for MultiPolygon<T> {
    fn map_coords_in_place_mut(&mut self, mut func: impl FnMut(Coord<T>) -> Coord<T>) {
        for p in &mut self.0 {
            p.map_coords_in_place_mut(&mut func);
        }
    }
}
impl<T: CoordNum> MapCoordsInPlaceMut<T> for GeometryCollection<T> {
    fn map_coords_in_place_mut(&mut self, mut func: impl FnMut(Coord<T>) -> Coord<T>) {
        for g in &mut self.0 {
            g.map_coords_in_place_mut(&mut func);
        }
    }
}
impl<T: CoordNum> MapCoordsInPlaceMut<T> for Geometry<T> {
    fn map_coords_in_place_mut(&mut self, func: impl FnMut(Coord<T>) -> Coord<T>) {
        match *self {
            Geometry::Point(ref mut x) => x.map_coords_in_place_mut(func),
            Geometry::Line(ref mut x) => x.map_coords_in_place_mut(func),
            Geometry::LineString(ref mut x) => x.map_coords_in_place_mut(func),
            Geometry::Polygon(ref mut x) => x.map_coords_in_place_mut(func),
            Geometry::MultiPoint(ref mut x) => x.map_coords_in_place_mut(func),
            Geometry::MultiLineString(ref mut x) => x.map_coords_in_place_mut(func),
            Geometry::MultiPolygon(ref mut x) => x.map_coords_in_place_mut(func),
            Geometry::Rect(ref mut x) => x.map_coords_in_place_mut(func),
            Geometry::Triangle(ref mut x) => x.map_coords_in_place_mut(func),
            Geometry::GeometryCollection(ref mut _x) => {
                // x.map_coords_in_place_mut(func)
                unimplemented!("Mapping coordinates for GEOMETRYCOLLECTIONs is unsupported");
                // Mapping a GeometryCollection requires calling map_coords_in_place_mut on
                // whatever Geometry the collection contains, which might very well be another
                // GeometryCollection (however unlikely).
                //
                // TODO: Add a unbundler to the tools/bundle.rs tool.
                //
                // error: reached the recursion limit while instantiating `<Geometry as MapCoordsInPlaceMut<f64>>::map_coords_in_place_mut::<&mut &mut &mut ...>`
                //   --> generative/geometry_mut_map.rs:81:13
                //    |
                // 81 |             g.map_coords_in_place_mut(&mut func);
                //    |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
                //    |
                // note: `<Geometry<T> as MapCoordsInPlaceMut<T>>::map_coords_in_place_mut` defined here
                //   --> generative/geometry_mut_map.rs:86:5
                //    |
                // 86 |     fn map_coords_in_place_mut(&mut self, func: impl FnMut(Coord<T>) -> Coord<T>) {
                //    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
                //    = note: the full type name has been written to 'target/debug/deps/streamline-a2b6c690ece9d693.long-type.txt'
                //
                // <Geometry as MapCoordsInPlaceMut<f64>>::map_coords_in_place_mut::<&mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut &mut [closure@tools/streamline.rs:378:38: 378:45]>
            }
        }
    }
}
