#pragma once
#include <geos/geom/Geometry.h>

#include <iterator>

namespace geom2graph {

//! @brief Recursively flatten geometries.
class GeometryFlattener
{
    struct RecursiveGeometryIterator :
        //! @todo Use forward_iterator_tag
        public std::iterator<std::input_iterator_tag, geos::geom::Geometry>
    {
        RecursiveGeometryIterator(const geos::geom::Geometry& geometry, int n);

        const value_type& operator*() const;
        const value_type* operator->() const;
        RecursiveGeometryIterator& operator++();
        // RecursiveGeometryIterator& operator++(int);
        bool operator==(const RecursiveGeometryIterator& rhs) const;
        bool operator!=(const RecursiveGeometryIterator& rhs) const;
        explicit operator bool() const;

    private:
        const value_type& m_geometry;
        // Here's the recursion.
        std::unique_ptr<RecursiveGeometryIterator> m_iterator;
        const size_t m_num_subgeometries;
        size_t m_index;
        bool m_exhausted;
    };

public:
    using const_iterator = RecursiveGeometryIterator;
    using iterator = const_iterator;
    GeometryFlattener(const geos::geom::Geometry& geometry);

    [[nodiscard]] const_iterator cbegin() const { return const_iterator(m_geometry, 0); };
    [[nodiscard]] const_iterator cend() const { return const_iterator(m_geometry, -1); };
    
    [[nodiscard]] iterator begin() const { return iterator(m_geometry, 0); };
    [[nodiscard]] iterator end() const { return iterator(m_geometry, -1); };

private:
    const geos::geom::Geometry& m_geometry;
};
}  // namespace geom2graph
