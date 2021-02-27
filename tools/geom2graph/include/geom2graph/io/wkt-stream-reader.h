#pragma once
#include <geos/io/WKTReader.h>

#include <istream>
#include <iterator>
#include <memory>

namespace geos::geom {
class Geometry;
class GeometryFactory;
}  // namespace geos::geom

namespace geom2graph::io {

//! @brief Deserialize geometries from a WKT input stream.
//! @details Wraps a std::istream providing WKT geometries, one per line, and provides an iterator
//! interface to consume the istream, and yield one geometry after another.
class WKTStreamReader
{
    struct GeometryIterator :
        public std::iterator<std::input_iterator_tag, std::unique_ptr<geos::geom::Geometry>>
    {
        GeometryIterator(std::istream& input_stream,
                         const geos::geom::GeometryFactory& factory,
                         bool is_done = false);

        reference operator*() { return m_current_value; }
        pointer operator->() { return &m_current_value; }
        GeometryIterator& operator++();
        bool operator==(const GeometryIterator& rhs) const;
        bool operator!=(const GeometryIterator& rhs) const;
        explicit operator bool() const { return !m_is_past_end; }

    private:
        bool m_is_at_end;
        bool m_is_past_end;
        std::istream& m_input_stream;
        value_type m_current_value;
        geos::io::WKTReader m_wkt_reader;
    };

public:
    using iterator = GeometryIterator;
    explicit WKTStreamReader(std::istream& input_stream) :
        m_input(input_stream), m_factory(geos::geom::GeometryFactory::create())
    {
    }
    WKTStreamReader(std::istream& input_stream, geos::geom::GeometryFactory::Ptr factory) :
        m_input(input_stream), m_factory(std::move(factory))
    {
    }

    [[nodiscard]] iterator begin() const { return iterator(m_input, *m_factory); }
    [[nodiscard]] iterator end() const { return iterator(m_input, *m_factory, true); }

private:
    std::istream& m_input;
    geos::geom::GeometryFactory::Ptr m_factory;
};
}  // namespace geom2graph::io
