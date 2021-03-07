#pragma once
#include "geom2graph/noding/geometry-graph.h"

#include <geos/io/WKTReader.h>

#include <istream>

namespace geom2graph::io {
class GraphReader
{
public:
    GraphReader(std::istream& input, const geos::geom::GeometryFactory& factory) :
        m_input(input), m_factory(factory), m_reader(factory)
    {
    }
    virtual ~GraphReader() = default;

    //! @brief Read a graph from the given istream.
    virtual geom2graph::noding::GeometryGraph read() noexcept = 0;
    virtual void read(const std::string& line) noexcept = 0;

protected:
    std::istream& m_input;
    const geos::geom::GeometryFactory& m_factory;
    geos::io::WKTReader m_reader;
};
}  // namespace geom2graph::io
