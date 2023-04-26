#pragma once
#include <geos/geom/Geometry.h>

#include <cstddef>
#include <iterator>

namespace generative {

//! @brief Recursively flatten geometries.
class GeometryFlattener
{
    struct RecursiveGeometryIterator
    {
        using iterator_category = std::input_iterator_tag;
        using value_type = geos::geom::Geometry;
        using difference_type = std::ptrdiff_t;
        using pointer = value_type*;
        using reference = value_type&;

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

    [[nodiscard]] const_iterator cbegin() const { return {m_geometry, 0}; };
    [[nodiscard]] const_iterator cend() const { return {m_geometry, -1}; };

    [[nodiscard]] iterator begin() const { return {m_geometry, 0}; };
    [[nodiscard]] iterator end() const { return {m_geometry, -1}; };

private:
    const geos::geom::Geometry& m_geometry;
};
}  // namespace generative
