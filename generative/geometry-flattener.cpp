#include "generative/geometry-flattener.h"

#include <log4cplus/logger.h>
#include <log4cplus/loggingmacros.h>

static auto s_logger = log4cplus::Logger::getInstance("generative.geometryflattener");

static bool is_multi_geometry(const geos::geom::Geometry* geometry)
{
    switch (geometry->getGeometryTypeId())
    {
    case geos::geom::GeometryTypeId::GEOS_CIRCULARSTRING:
    case geos::geom::GeometryTypeId::GEOS_CURVEPOLYGON:
    case geos::geom::GeometryTypeId::GEOS_LINEARRING:
    case geos::geom::GeometryTypeId::GEOS_LINESTRING:
    case geos::geom::GeometryTypeId::GEOS_POINT:
    case geos::geom::GeometryTypeId::GEOS_POLYGON:
        return false;

    case geos::geom::GeometryTypeId::GEOS_COMPOUNDCURVE:
    case geos::geom::GeometryTypeId::GEOS_GEOMETRYCOLLECTION:
    case geos::geom::GeometryTypeId::GEOS_MULTICURVE:
    case geos::geom::GeometryTypeId::GEOS_MULTILINESTRING:
    case geos::geom::GeometryTypeId::GEOS_MULTIPOINT:
    case geos::geom::GeometryTypeId::GEOS_MULTIPOLYGON:
    case geos::geom::GeometryTypeId::GEOS_MULTISURFACE:
        return true;
    }
    return false;
}

namespace generative {
GeometryFlattener::GeometryFlattener(const geos::geom::Geometry& geometry) : m_geometry(geometry)
{
}

GeometryFlattener::RecursiveGeometryIterator::RecursiveGeometryIterator(
    const geos::geom::Geometry& geometry, int n) :
    m_geometry(geometry),
    m_iterator(nullptr),
    m_num_subgeometries(m_geometry.getNumGeometries()),
    m_index(n < 0 ? m_num_subgeometries : static_cast<size_t>(n)),
    m_exhausted(m_index >= m_num_subgeometries)
{
    if (!m_exhausted && !m_iterator && is_multi_geometry(m_geometry.getGeometryN(m_index)))
    {
        LOG4CPLUS_TRACE(s_logger,
                        "Initializing " << m_geometry.getGeometryType()
                                        << " iterator with new recursive iterator for child "
                                        << m_geometry.getGeometryN(m_index)->getGeometryType());
        m_iterator = std::make_unique<GeometryFlattener::RecursiveGeometryIterator>(
            *m_geometry.getGeometryN(m_index), 0);
    }
}

const geos::geom::Geometry& GeometryFlattener::RecursiveGeometryIterator::operator*() const
{
    if (!m_exhausted)
    {
        if (m_iterator)
        {
            return m_iterator->operator*();
        }
        return *m_geometry.getGeometryN(m_index);
    }
    return m_geometry;
}

const geos::geom::Geometry* GeometryFlattener::RecursiveGeometryIterator::operator->() const
{
    if (!m_exhausted)
    {
        if (m_iterator)
        {
            return m_iterator->operator->();
        }
        return m_geometry.getGeometryN(m_index);
    }
    return &m_geometry;
}

GeometryFlattener::RecursiveGeometryIterator&
GeometryFlattener::RecursiveGeometryIterator::operator++()
{
    // We're a multi-geometry, so get the next child.
    if (m_iterator)
    {
        auto& iter = *m_iterator;
        LOG4CPLUS_TRACE(s_logger,
                        "Advancing child " << m_iterator->m_geometry.getGeometryType()
                                           << "'s iterator");
        if (!iter.m_exhausted)
        {
            ++iter;
        }

        // The child iterator has been exhausted, now move to the next child.
        if (iter.m_exhausted)
        {
            m_index++;
            m_iterator.reset(nullptr);
        }
    } else
    {
        LOG4CPLUS_TRACE(s_logger,
                        "Advancing " << m_geometry.getGeometryType()
                                     << "'s iterator to next geometry");
        m_index++;
    }
    m_exhausted = m_index >= m_num_subgeometries;
    if (m_exhausted)
    {
        LOG4CPLUS_TRACE(s_logger, "Iterator for " << m_geometry.getGeometryType() << " exhausted");
        return *this;
    }

    // Check if the current geometry is a collection, and if it is, create an iterator to step
    // through its children.
    if (!m_iterator && is_multi_geometry(m_geometry.getGeometryN(m_index)))
    {
        LOG4CPLUS_TRACE(s_logger,
                        "Creating new recursive iterator for child "
                            << m_geometry.getGeometryN(m_index)->getGeometryType());
        m_iterator = std::make_unique<GeometryFlattener::RecursiveGeometryIterator>(
            *m_geometry.getGeometryN(m_index), 0);
    }

    return *this;
}

bool GeometryFlattener::RecursiveGeometryIterator::operator==(
    const RecursiveGeometryIterator& rhs) const
{
    // They need to both have iterators, or both not have iterators.
    const bool children_iter_xor = !m_iterator != !rhs.m_iterator;
    bool children_equal = true;
    if (m_iterator && !children_iter_xor)
    {
        children_equal = m_iterator->operator==(*rhs.m_iterator);
    }
    return (&this->m_geometry == &rhs.m_geometry) && this->m_index == rhs.m_index && children_equal;
}

bool GeometryFlattener::RecursiveGeometryIterator::operator!=(
    const RecursiveGeometryIterator& rhs) const
{
    return !(*this == rhs);
}

GeometryFlattener::RecursiveGeometryIterator::operator bool() const
{
    return !m_exhausted;
}
}  // namespace generative
