#pragma once
#include <memory>

namespace geos::geom {
    class Geometry;
}  // namespace geos::geom

namespace geom2graph::io {
    //! @brief Reads a single geometry from the given WKT text.
    //! @returns nullptr if the WKT is invalid.
    std::unique_ptr<geos::geom::Geometry> from_wkt(const std::string& wkt);
}  // namespace geom2graph::io
