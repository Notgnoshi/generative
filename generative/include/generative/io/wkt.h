#pragma once
#include <memory>
#include <ostream>

namespace geos::geom {
class Geometry;
class Point;
}  // namespace geos::geom

namespace generative::io {
//! @brief Reads a single geometry from the given WKT text.
//! @returns nullptr if the WKT is invalid.
std::unique_ptr<geos::geom::Geometry> from_wkt(const std::string& wkt);

std::ostream& operator<<(std::ostream& out, const std::unique_ptr<geos::geom::Geometry>& geom);
std::ostream& operator<<(std::ostream& out, const std::unique_ptr<geos::geom::Point>& point);
std::ostream& operator<<(std::ostream& out, const geos::geom::Geometry& geom);
}  // namespace generative::io
