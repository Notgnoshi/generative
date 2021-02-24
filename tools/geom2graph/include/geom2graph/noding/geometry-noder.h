#pragma once

#include <geos/noding/SegmentString.h>
#include <memory.h>

namespace geos {
namespace geom {
    class Geometry;
}
namespace noding {
    class Noder;
}
}  // namespace geos

namespace geom2graph::noding {

//! @brief Nodes a geometry using the specified geos::noding::Noder.
//! @note The process of "noding" a geometry is finding all pairs intersections, and breaking the
//! geometry into a sequence of non-intersecting (except at the endpoints)
//! geos::noding::SegmentString's.
//! @details The implementation of geom2graph::noding::GeometryNoder is the same as
//! geos::noding::GeometryNoder, with the sole exception that you can provide your own
//! geos::noding::Noder.
//! @see geos::noding::GeometryNoder
class GeometryNoder
{
public:
    //! @todo Provide a geos::noding::Noder. Use template? Or pass a unique_ptr?
    GeometryNoder(const geos::geom::Geometry& geometry);
    GeometryNoder(GeometryNoder const&) = delete;
    GeometryNoder& operator=(GeometryNoder const&) = delete;

    std::unique_ptr<geos::geom::Geometry> get_noded();
    static std::unique_ptr<geos::geom::Geometry> node(const geos::geom::Geometry& geometry);

private:
    const geos::geom::Geometry& m_geometry;
    geos::noding::SegmentString::NonConstVect m_lines;
    std::unique_ptr<geos::noding::Noder> m_noder;

    static void extract_segment_strings(const geos::geom::Geometry& geometry,
                                        geos::noding::SegmentString::NonConstVect& out);
    geos::noding::Noder& get_noder();
    std::unique_ptr<geos::geom::Geometry>
    to_geometry(geos::noding::SegmentString::NonConstVect& noded);
};
}  // namespace geom2graph::noding
