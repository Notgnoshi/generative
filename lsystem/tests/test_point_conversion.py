import unittest

import numpy as np
from lsystem.projection import PointConversion, PointTag
from numpy.testing import assert_allclose
from shapely.geometry import (
    GeometryCollection,
    LineString,
    MultiLineString,
    MultiPoint,
    MultiPolygon,
    Point,
    Polygon,
)


class TestToTaggedPoints(unittest.TestCase):
    def test_wrap_sequence(self):
        sequence = [
            ("p", None),
            ("p", None),
        ]
        expected = [("p", PointTag.POLYGON_BEGIN), ("p", PointTag.POLYGON_END)]
        wrapped = list(PointConversion._wrap_sequence(iter(sequence), PointTag.POLYGON_BEGIN))
        for actual, desired in zip(wrapped, expected):
            self.assertTupleEqual(actual, desired)

        sequence = [
            ("p", None),
            ("p", None),
            ("p", None),
        ]
        expected = [
            ("p", PointTag.POLYGON_BEGIN),
            ("p", None),
            ("p", PointTag.POLYGON_END),
        ]

        wrapped = list(PointConversion._wrap_sequence(iter(sequence), PointTag.POLYGON_BEGIN))
        for actual, desired in zip(wrapped, expected):
            self.assertTupleEqual(actual, desired)

        sequence = [
            ("p", PointTag.LINESTRING_BEGIN),
            ("p", None),
            ("p", PointTag.LINESTRING_END),
        ]
        expected = [
            ("p", (PointTag.POLYGON_BEGIN, PointTag.LINESTRING_BEGIN)),
            ("p", None),
            ("p", (PointTag.LINESTRING_END, PointTag.POLYGON_END)),
        ]

        wrapped = list(PointConversion._wrap_sequence(iter(sequence), PointTag.POLYGON_BEGIN))
        for actual, desired in zip(wrapped, expected):
            self.assertTupleEqual(actual, desired)

    def test_point(self):
        p = Point(0, 1)
        tagged = list(PointConversion._to_points(p))

        self.assertEqual(len(tagged), 1)
        self.assertTupleEqual(tagged[0], (p.coords[0], None))

    def test_points(self):
        points = [Point(0, 1), Point(2, 3)]
        tagged = list(PointConversion.to_points(points))

        self.assertEqual(len(tagged), 2)

        for point, tagged_point in zip(points, tagged):
            self.assertTupleEqual(tagged_point, (point.coords[0], None))

    def test_multipoint(self):
        m = MultiPoint([(0, 1), (2, 3), (4, 5), (6, 7)])
        tagged = list(PointConversion._to_points(m))

        self.assertEqual(len(tagged), len(m))

        expected = [
            (m[0].coords[0], PointTag.MULTIPOINT_BEGIN),
            (m[1].coords[0], None),
            (m[2].coords[0], None),
            (m[3].coords[0], PointTag.MULTIPOINT_END),
        ]
        for actual, desired in zip(tagged, expected):
            self.assertTupleEqual(actual, desired)

    def test_linestring(self):
        l = LineString([(0, 1), (2, 3), (4, 5)])
        tagged = list(PointConversion._to_points(l))
        expected = [
            (l.coords[0], PointTag.LINESTRING_BEGIN),
            (l.coords[1], None),
            (l.coords[2], PointTag.LINESTRING_END),
        ]
        for actual, desired in zip(tagged, expected):
            self.assertTupleEqual(actual, desired)

    def test_linestrings(self):
        ls = [
            LineString([(0, 1), (2, 3), (4, 5)]),
            LineString([(6, 7), (8, 9)]),
        ]
        tagged = list(PointConversion.to_points(ls))
        expected = [
            (ls[0].coords[0], PointTag.LINESTRING_BEGIN),
            (ls[0].coords[1], None),
            (ls[0].coords[2], PointTag.LINESTRING_END),
            (ls[1].coords[0], PointTag.LINESTRING_BEGIN),
            (ls[1].coords[1], PointTag.LINESTRING_END),
        ]
        for actual, desired in zip(tagged, expected):
            self.assertTupleEqual(actual, desired)

    def test_multilinestring(self):
        ls = [
            LineString([(0, 1), (2, 3), (4, 5)]),
            LineString([(6, 7), (8, 9)]),
        ]
        ml = MultiLineString(lines=ls)

        tagged = list(PointConversion._to_points(ml))
        expected = [
            (ls[0].coords[0], (PointTag.MULTILINESTRING_BEGIN, PointTag.LINESTRING_BEGIN)),
            (ls[0].coords[1], None),
            (ls[0].coords[2], PointTag.LINESTRING_END),
            (ls[1].coords[0], PointTag.LINESTRING_BEGIN),
            (ls[1].coords[1], (PointTag.LINESTRING_END, PointTag.MULTILINESTRING_END)),
        ]

        for actual, desired in zip(tagged, expected):
            self.assertTupleEqual(actual, desired)

    def test_polygon_no_holes(self):
        p = Polygon(shell=[(0, 1), (2, 3), (4, 5)])
        tagged = list(PointConversion._to_points(p))
        expected = [
            (p.exterior.coords[0], (PointTag.POLYGON_BEGIN, PointTag.SHELL_BEGIN)),
            (p.exterior.coords[1], None),
            (p.exterior.coords[2], None),
            (p.exterior.coords[0], (PointTag.SHELL_END, PointTag.POLYGON_END)),
        ]
        self.assertEqual(len(tagged), 4)  # rings share the same begin and end point
        for actual, desired in zip(tagged, expected):
            self.assertTupleEqual(actual, desired)

    def test_polygon_hole(self):
        p = Polygon(shell=[(0, 1), (2, 3), (4, 5)], holes=[[(6, 7), (8, 9), (10, 11)]])
        tagged = list(PointConversion._to_points(p))
        expected = [
            (p.exterior.coords[0], (PointTag.POLYGON_BEGIN, PointTag.SHELL_BEGIN)),
            (p.exterior.coords[1], None),
            (p.exterior.coords[2], None),
            (p.exterior.coords[0], PointTag.SHELL_END),
            (p.interiors[0].coords[0], PointTag.HOLE_BEGIN),
            (p.interiors[0].coords[1], None),
            (p.interiors[0].coords[2], None),
            (p.interiors[0].coords[0], (PointTag.HOLE_END, PointTag.POLYGON_END)),
        ]
        self.assertEqual(len(tagged), 8)  # rings share the same begin and end point
        for actual, desired in zip(tagged, expected):
            self.assertTupleEqual(actual, desired)

    def test_polygons(self):
        p1 = Polygon(shell=[(0, 1), (2, 3), (4, 5)])
        p2 = Polygon(
            shell=[(0, 1), (2, 3), (4, 5)],
            holes=[[(6, 7), (8, 9), (10, 11)], [(12, 13), (14, 15), (16, 17)]],
        )
        tagged = list(PointConversion.to_points([p1, p2]))
        expected = [
            (p1.exterior.coords[0], (PointTag.POLYGON_BEGIN, PointTag.SHELL_BEGIN)),
            (p1.exterior.coords[1], None),
            (p1.exterior.coords[2], None),
            (p1.exterior.coords[0], (PointTag.SHELL_END, PointTag.POLYGON_END)),
            (p2.exterior.coords[0], (PointTag.POLYGON_BEGIN, PointTag.SHELL_BEGIN)),
            (p2.exterior.coords[1], None),
            (p2.exterior.coords[2], None),
            (p2.exterior.coords[0], PointTag.SHELL_END),
            (p2.interiors[0].coords[0], PointTag.HOLE_BEGIN),
            (p2.interiors[0].coords[1], None),
            (p2.interiors[0].coords[2], None),
            (p2.interiors[0].coords[0], PointTag.HOLE_END),
            (p2.interiors[1].coords[0], PointTag.HOLE_BEGIN),
            (p2.interiors[1].coords[1], None),
            (p2.interiors[1].coords[2], None),
            (p2.interiors[1].coords[0], (PointTag.HOLE_END, PointTag.POLYGON_END)),
        ]
        self.assertEqual(len(tagged), 16)
        for actual, desired in zip(tagged, expected):
            self.assertTupleEqual(actual, desired)

    def test_multipolygon(self):
        p1 = Polygon(shell=[(0, 1), (2, 3), (4, 5)])
        p2 = Polygon(
            shell=[(0, 1), (2, 3), (4, 5)],
            holes=[[(6, 7), (8, 9), (10, 11)], [(12, 13), (14, 15), (16, 17)]],
        )
        tagged = list(PointConversion._to_points(MultiPolygon([p1, p2])))
        expected = [
            (
                p1.exterior.coords[0],
                (PointTag.MULTIPOLYGON_BEGIN, PointTag.POLYGON_BEGIN, PointTag.SHELL_BEGIN),
            ),
            (p1.exterior.coords[1], None),
            (p1.exterior.coords[2], None),
            (p1.exterior.coords[0], (PointTag.SHELL_END, PointTag.POLYGON_END)),
            (p2.exterior.coords[0], (PointTag.POLYGON_BEGIN, PointTag.SHELL_BEGIN)),
            (p2.exterior.coords[1], None),
            (p2.exterior.coords[2], None),
            (p2.exterior.coords[0], PointTag.SHELL_END),
            (p2.interiors[0].coords[0], PointTag.HOLE_BEGIN),
            (p2.interiors[0].coords[1], None),
            (p2.interiors[0].coords[2], None),
            (p2.interiors[0].coords[0], PointTag.HOLE_END),
            (p2.interiors[1].coords[0], PointTag.HOLE_BEGIN),
            (p2.interiors[1].coords[1], None),
            (p2.interiors[1].coords[2], None),
            (
                p2.interiors[1].coords[0],
                (PointTag.HOLE_END, PointTag.POLYGON_END, PointTag.MULTIPOLYGON_END),
            ),
        ]
        self.assertEqual(len(tagged), 16)
        for actual, desired in zip(tagged, expected):
            self.assertTupleEqual(actual, desired)

    def test_geometry_collection(self):
        p1 = Polygon(shell=[(0, 1), (2, 3), (4, 5)])
        p2 = Polygon(
            shell=[(0, 1), (2, 3), (4, 5)],
            holes=[[(6, 7), (8, 9), (10, 11)], [(12, 13), (14, 15), (16, 17)]],
        )
        tagged = list(
            PointConversion._to_points(
                GeometryCollection(
                    [Point(0, 0), MultiPolygon([p1, p2]), LineString([(0, 0), (1, 1)])]
                )
            )
        )
        expected = [
            ((0, 0), PointTag.COLLECTION_BEGIN),
            (
                p1.exterior.coords[0],
                (PointTag.MULTIPOLYGON_BEGIN, PointTag.POLYGON_BEGIN, PointTag.SHELL_BEGIN),
            ),
            (p1.exterior.coords[1], None),
            (p1.exterior.coords[2], None),
            (p1.exterior.coords[0], (PointTag.SHELL_END, PointTag.POLYGON_END)),
            (p2.exterior.coords[0], (PointTag.POLYGON_BEGIN, PointTag.SHELL_BEGIN)),
            (p2.exterior.coords[1], None),
            (p2.exterior.coords[2], None),
            (p2.exterior.coords[0], PointTag.SHELL_END),
            (p2.interiors[0].coords[0], PointTag.HOLE_BEGIN),
            (p2.interiors[0].coords[1], None),
            (p2.interiors[0].coords[2], None),
            (p2.interiors[0].coords[0], PointTag.HOLE_END),
            (p2.interiors[1].coords[0], PointTag.HOLE_BEGIN),
            (p2.interiors[1].coords[1], None),
            (p2.interiors[1].coords[2], None),
            (
                p2.interiors[1].coords[0],
                (PointTag.HOLE_END, PointTag.POLYGON_END, PointTag.MULTIPOLYGON_END),
            ),
            ((0, 0), PointTag.LINESTRING_BEGIN),
            ((1, 1), (PointTag.LINESTRING_END, PointTag.COLLECTION_END)),
        ]
        self.assertEqual(len(tagged), 19)
        for actual, desired in zip(tagged, expected):
            self.assertTupleEqual(actual, desired)

    def test_nested_geometry_collection(self):
        geoms = GeometryCollection(
            [GeometryCollection([GeometryCollection([Point(0, 0), Point(1, 1)])])]
        )
        tagged = list(PointConversion._to_points(geoms))
        expected = [
            (
                (0, 0),
                (
                    PointTag.COLLECTION_BEGIN,
                    PointTag.COLLECTION_BEGIN,
                    PointTag.COLLECTION_BEGIN,
                ),
            ),
            (
                (1, 1),
                (
                    PointTag.COLLECTION_END,
                    PointTag.COLLECTION_END,
                    PointTag.COLLECTION_END,
                ),
            ),
        ]
        self.assertEqual(len(tagged), 2)
        for actual, desired in zip(tagged, expected):
            self.assertTupleEqual(actual, desired)


class TestFromTaggedPoints(unittest.TestCase):
    def test_points(self):
        points = [
            Point(0, 1),
            Point(2, 3),
            Point(4, 5),
        ]
        tagged = list(PointConversion.to_points(points))
        expected = [
            ((0, 1), None),
            ((2, 3), None),
            ((4, 5), None),
        ]
        self.assertEqual(len(tagged), 3)
        for actual, desired in zip(tagged, expected):
            self.assertTupleEqual(actual, desired)

        geoms = list(PointConversion.from_points(tagged))

        self.assertEqual(len(geoms), 3)
        for actual, desired in zip(geoms, points):
            self.assertEqual(actual, desired)

    def test_points_linestring(self):
        geometries = [Point(0, 1), Point(2, 3), Point(4, 5), LineString([(6, 7), (8, 9), (10, 11)])]
        tagged = list(PointConversion.to_points(geometries))
        new_geometries = list(PointConversion.from_points(tagged))

        self.assertEqual(len(new_geometries), 4)
        for actual, desired in zip(new_geometries, geometries):
            self.assertEqual(actual, desired)

    def test_polygon_no_holes(self):
        p = Polygon(shell=[(0, 1), (2, 3), (4, 5)])
        tagged = list(PointConversion.to_points([p]))
        new_geometries = list(PointConversion.from_points(tagged))

        self.assertEqual(len(new_geometries), 1)
        for actual, desired in zip(new_geometries, [p]):
            self.assertEqual(actual, desired)

    def test_polygon_holes(self):
        p = Polygon(shell=[(0, 1), (2, 3), (4, 5)], holes=[[(6, 7), (8, 9), (10, 11)]])
        tagged = list(PointConversion.to_points([p]))
        new_geometries = list(PointConversion.from_points(tagged))

        self.assertEqual(len(new_geometries), 1)
        for actual, desired in zip(new_geometries, [p]):
            self.assertEqual(actual, desired)

    def test_polygons(self):
        p1 = Polygon(shell=[(0, 1), (2, 3), (4, 5)])
        p2 = Polygon(
            shell=[(0, 1), (2, 3), (4, 5)],
            holes=[[(6, 7), (8, 9), (10, 11)], [(12, 13), (14, 15), (16, 17)]],
        )
        tagged = list(PointConversion.to_points([p1, p2]))
        new_geometries = list(PointConversion.from_points(tagged))

        self.assertEqual(len(new_geometries), 2)
        for actual, desired in zip(new_geometries, [p1, p2]):
            self.assertEqual(actual, desired)

    def test_multipoint(self):
        geometries = [
            Point(1, 1),
            MultiPoint([(0, 1), (2, 3), (4, 5), (6, 7)]),
            Point(8, 8),
        ]
        tagged = list(PointConversion.to_points(geometries))
        new_geometries = list(PointConversion.from_points(tagged))

        self.assertEqual(len(new_geometries), 3)
        for actual, desired in zip(new_geometries, geometries):
            self.assertEqual(actual, desired)

    def test_multilinestring(self):
        geometries = [
            LineString([(0, 0), (1, 1)]),
            MultiLineString(
                [
                    LineString([(0, 1), (2, 3), (4, 5)]),
                    LineString([(6, 7), (8, 9)]),
                ]
            ),
            LineString([(1, 1), (2, 2)]),
        ]
        tagged = list(PointConversion.to_points(geometries))
        new_geometries = list(PointConversion.from_points(tagged))

        self.assertEqual(len(new_geometries), 3)
        for actual, desired in zip(new_geometries, geometries):
            self.assertEqual(actual, desired)

    def test_multipolygon(self):
        geometries = [
            Point(0, 0),
            MultiPolygon(
                [
                    Polygon(shell=[(0, 1), (2, 3), (4, 5)]),
                    Polygon(
                        shell=[(0, 1), (2, 3), (4, 5)],
                        holes=[[(6, 7), (8, 9), (10, 11)], [(12, 13), (14, 15), (16, 17)]],
                    ),
                ]
            ),
            Point(1, 1),
        ]
        tagged = list(PointConversion.to_points(geometries))
        new_geometries = list(PointConversion.from_points(tagged))

        self.assertEqual(len(new_geometries), 3)
        for actual, desired in zip(new_geometries, geometries):
            self.assertEqual(actual, desired)

    def test_geometry_collection(self):
        geometries = [
            Point(0, 0),
            GeometryCollection(
                [
                    Polygon(shell=[(0, 1), (2, 3), (4, 5)]),
                    Polygon(
                        shell=[(0, 1), (2, 3), (4, 5)],
                        holes=[[(6, 7), (8, 9), (10, 11)], [(12, 13), (14, 15), (16, 17)]],
                    ),
                ]
            ),
            Point(1, 1),
        ]

        tagged = list(PointConversion.to_points(geometries))
        new_geometries = list(PointConversion.from_points(tagged))

        self.assertEqual(len(new_geometries), 3)
        for actual, desired in zip(new_geometries, geometries):
            self.assertEqual(actual, desired)

    def test_nested_geometry_collection(self):
        geometries = [
            Point(0, 0),
            GeometryCollection([GeometryCollection([Point(10, 10), Point(20, 20)])]),
            Point(1, 1),
        ]
        tagged = list(PointConversion.to_points(geometries))
        new_geometries = list(PointConversion.from_points(tagged))

        self.assertEqual(len(new_geometries), 3)
        for actual, desired in zip(new_geometries, geometries):
            self.assertEqual(actual, desired)

    def test_nested_geometry_collection_dos(self):
        geometries = [
            Point(0, 0),
            GeometryCollection(
                [
                    GeometryCollection(
                        [
                            Point(10, 10),
                            Polygon(
                                shell=[(0, 0), (1, 1), (2, 2)], holes=[[(1, 1), (2, 2), (3, 3)]]
                            ),
                            Point(20, 20),
                        ]
                    )
                ]
            ),
            Point(1, 1),
        ]
        tagged = list(PointConversion.to_points(geometries))
        new_geometries = list(PointConversion.from_points(tagged))

        self.assertEqual(len(new_geometries), 3)
        for actual, desired in zip(new_geometries, geometries):
            self.assertEqual(actual, desired)
