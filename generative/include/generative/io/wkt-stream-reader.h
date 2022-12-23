#pragma once
#include <geos/io/WKTReader.h>

#include <istream>
#include <iterator>
#include <memory>
#include <vector>

namespace geos::geom {
class Geometry;
class GeometryFactory;
}  // namespace geos::geom

namespace generative::io {

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
        m_input(input_stream),
        m_factory(geos::geom::GeometryFactory::create()),
        m_factory_ref(*m_factory)
    {
    }
    WKTStreamReader(std::istream& input_stream, geos::geom::GeometryFactory& factory) :
        m_input(input_stream), m_factory(nullptr), m_factory_ref(factory)
    {
    }

    [[nodiscard]] iterator begin() const { return iterator(m_input, m_factory_ref); }
    [[nodiscard]] iterator end() const { return iterator(m_input, m_factory_ref, true); }

    //! @brief Consume the input stream, and collapse into a std::vector of geometries.
    [[nodiscard]] std::vector<std::unique_ptr<geos::geom::Geometry>> collapse() const
    {
        std::vector<std::unique_ptr<geos::geom::Geometry>> geometries;
        // Can't reserve space ahead of time because we don't know how long the geometry stream is.

        for (auto iter = this->begin(); iter != this->end(); ++iter)
        {
            geometries.push_back(std::move(*iter));
        }

        return geometries;
    }

private:
    std::istream& m_input;
    geos::geom::GeometryFactory::Ptr m_factory;
    geos::geom::GeometryFactory& m_factory_ref;
};
}  // namespace generative::io
