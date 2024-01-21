#include "generative/noding/geometry-noder.h"

#include <geos/geom/Coordinate.h>
#include <geos/geom/Geometry.h>
#include <geos/geom/GeometryComponentFilter.h>
#include <geos/geom/GeometryFactory.h>
#include <geos/noding/IteratedNoder.h>
#include <geos/noding/NodedSegmentString.h>
#include <geos/noding/Noder.h>
#include <geos/noding/OrientedCoordinateArray.h>
#include <geos/noding/SegmentString.h>

#include <set>

namespace generative::noding {

class SegmentStringExtractor : public geos::geom::GeometryComponentFilter
{
public:
    SegmentStringExtractor(geos::noding::SegmentString::NonConstVect& to) : m_to(to) {}
    SegmentStringExtractor(const SegmentStringExtractor&) = delete;
    SegmentStringExtractor& operator=(const SegmentStringExtractor&) = delete;

    void filter_ro(const geos::geom::Geometry* geometry) override
    {
        switch (geometry->getGeometryTypeId())
        {
        case geos::geom::GeometryTypeId::GEOS_POINT:
        {
            // HACK: This is gross and dirty, and has the potential to explode in future GEOS
            // releases. Single-coordinate segment strings (which isn't _really_ a **segment**
            // string at all) don't get noded.
            //
            // So bold facedly lie to the noder, by treating POINTs as a "segment" with two
            // duplicate coordinates.
            auto coords = geometry->getCoordinates();
            coords->add(coords->getAt(0), true);
            geos::noding::SegmentString* segment_string =
                new geos::noding::NodedSegmentString(coords.release(), false, false, nullptr);
            m_to.push_back(segment_string);
            break;
        }
        case geos::geom::GeometryTypeId::GEOS_LINESTRING:
        case geos::geom::GeometryTypeId::GEOS_LINEARRING:
        {
            auto coords = geometry->getCoordinates();
            geos::noding::SegmentString* segment_string =
                new geos::noding::NodedSegmentString(coords.release(), false, false, nullptr);
            m_to.push_back(segment_string);
            break;
        }
        // need to get the interior holes too
        case geos::geom::GeometryTypeId::GEOS_POLYGON:
        {
            const auto* poly = dynamic_cast<const geos::geom::Polygon*>(geometry);
            if (poly != nullptr)
            {
                const auto* exterior = poly->getExteriorRing();
                this->filter_ro(exterior);

                for (size_t i = 0; i < poly->getNumInteriorRing(); i++)
                {
                    const auto* interior = poly->getInteriorRingN(i);
                    this->filter_ro(interior);
                }
            }
            break;
        }

        // Intentional fallthrough
        case geos::geom::GeometryTypeId::GEOS_MULTIPOINT:
        case geos::geom::GeometryTypeId::GEOS_MULTILINESTRING:
        case geos::geom::GeometryTypeId::GEOS_MULTIPOLYGON:
        case geos::geom::GeometryTypeId::GEOS_GEOMETRYCOLLECTION:
            // apply_ro automatically extracts each of the contained geometries
            break;
        }
    }

private:
    geos::noding::SegmentString::NonConstVect& m_to;
};

GeometryNoder::GeometryNoder(const geos::geom::Geometry& geometry,
                             std::unique_ptr<geos::noding::Noder> noder) :
    // If m_noder is null, get_noder will create a default IteratedNoder.
    m_geometry(geometry), m_noder(std::move(noder))
{
}

std::unique_ptr<geos::geom::Geometry> GeometryNoder::get_noded()
{
    geos::noding::SegmentString::NonConstVect lines;
    extract_segment_strings(m_geometry, lines);

    geos::noding::Noder& noder = get_noder();
    geos::noding::SegmentString::NonConstVect* noded_edges = nullptr;

    try
    {
        noder.computeNodes(&lines);
        noded_edges = noder.getNodedSubstrings();
        //! @todo Is there a more specific exception I can catch?
    } catch (const std::exception&)
    {
        for (size_t i = 0, n = lines.size(); i < n; ++i)
        {
            delete lines[i];
        }
        //! @todo what if I just return a nullptr?
        throw;
    }

    std::unique_ptr<geos::geom::Geometry> noded = to_geometry(*noded_edges);

    //! @todo Is there a way to do this automatically?
    //! @todo Is there a way to avoid these allocations in the first place?
    for (auto& edge : *noded_edges)
    {
        delete edge;
    }
    for (auto& line : lines)
    {
        delete line;
    }

    return noded;
}

std::unique_ptr<geos::geom::Geometry>
GeometryNoder::node(const geos::geom::Geometry& geometry,
                    std::unique_ptr<geos::noding::Noder> noder)
{
    GeometryNoder geom_noder(geometry, std::move(noder));
    return geom_noder.get_noded();
}
void GeometryNoder::extract_segment_strings(const geos::geom::Geometry& geometry,
                                            geos::noding::SegmentString::NonConstVect& out)
{
    SegmentStringExtractor extractor(out);
    geometry.apply_ro(&extractor);
}

geos::noding::Noder& GeometryNoder::get_noder()
{
    if (!m_noder)
    {
        const geos::geom::PrecisionModel* pm = m_geometry.getFactory()->getPrecisionModel();
        m_noder = std::make_unique<geos::noding::IteratedNoder>(pm);
    }

    return *m_noder;
}

std::unique_ptr<geos::geom::Geometry>
GeometryNoder::to_geometry(geos::noding::SegmentString::NonConstVect& noded_edges)
{
    const auto* geom_factory = m_geometry.getFactory();
    std::set<geos::noding::OrientedCoordinateArray> ocas;

    std::vector<std::unique_ptr<geos::geom::Geometry>> lines;
    lines.reserve(noded_edges.size());
    for (auto& ss : noded_edges)
    {
        auto* coords = ss->getCoordinates();

        // Check if an equivalent edge is known
        geos::noding::OrientedCoordinateArray oca(*coords);
        // NOTE: Each coordinate sequence is expected to be exactly two coordinates.
        if (ocas.insert(oca).second)
        {
            if (coords->front() == coords->back())
            {
                lines.push_back(geom_factory->createPoint(coords->front()));
            } else
            {
                lines.push_back(geom_factory->createLineString(coords->clone()));
            }
        }
    }

    return geom_factory->createGeometryCollection(std::move(lines));
}

}  // namespace generative::noding
