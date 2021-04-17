import io

import pytest
from shapely.geometry import (
    GeometryCollection,
    LineString,
    MultiLineString,
    MultiPoint,
    MultiPolygon,
    Point,
    Polygon,
)

from generative.flatten import flatten
from generative.wkio import deserialize_flat, deserialize_geometries, serialize_flat

test_geometries = [
    Point(0, 0),
    Polygon(shell=[(0, 0), (1, 1), (2, 2)]),
    LineString([(0, 0), (1, 1), (2, 2.02)]),
    MultiPoint([(0, 0), (1.0001, 1 / 3)]),
    MultiPolygon(
        [
            Polygon(shell=[(0, 0), (1, 0), (1, 1), (0, 1.00001)]),
            Polygon(shell=[(42, 0), (0, 42), (1056, 105)]),
        ]
    ),
    MultiLineString(
        [
            # 1 + 1/3 sucks due to floating point shenanigans
            LineString([(0, 0), (1 / 3, 1 + 3)]),
            LineString([(3, 3), (0.999, 2.3333)]),
        ]
    ),
    GeometryCollection(
        [
            Point(0, 0.1),
            LineString([(0, 0), (1, 1), (2, 2)]),
            MultiPoint([(0, 0), (5, 6)]),
            Polygon(
                shell=[(0, 0), (0, 10), (10, 10), (10, 0)],
                holes=[[(1, 1), (1, 2), (2, 2), (2, 1)]],
            ),
        ]
    ),
]


@pytest.mark.parametrize("geometry", test_geometries)
def test_wkt_serialize_deserialize(geometry):
    """Ensure that the geometry is unchanged after serialization and deserialization."""
    assert geometry == next(deserialize_geometries(io.StringIO(geometry.wkt), fmt="wkt"))


@pytest.mark.parametrize("geometry", test_geometries)
def test_wkb_serialize_deserialize(geometry):
    assert geometry == next(deserialize_geometries(io.StringIO(geometry.wkb_hex), fmt="wkb"))


@pytest.mark.parametrize("geometry", test_geometries)
def test_flat_serialize_deserialize(geometry):
    tagged_points = flatten([geometry])
    buffer = io.StringIO()
    serialize_flat(tagged_points, buffer)
    buffer.seek(0)
    assert geometry == next(deserialize_geometries(buffer, fmt="flat"))


def test_serialize_flat_points():
    expected = [
        "(0.0, 0.0)\n",
        "(1.0, 1.0)\n",
        "(2.0, 2.0, 2.0)\n",
    ]
    geoms = [Point(0, 0), Point(1, 1), Point(2, 2, 2)]
    output = io.StringIO()
    serialize_flat(flatten(geoms), output)
    output.seek(0)
    actual = output.readlines()
    assert actual == expected


def test_serialize_flat_linestring():
    expected = [
        "(0.0, 1.0)\tLINESTRING_BEGIN\n",
        "(2.0, 3.0)\n",
        "(4.0, 5.0)\tLINESTRING_END\n",
    ]
    geoms = [LineString([(0, 1), (2, 3), (4, 5)])]
    output = io.StringIO()
    serialize_flat(flatten(geoms), output)
    output.seek(0)
    actual = output.readlines()
    assert actual == expected


def test_serialize_flat_polygon():
    expected = [
        "(0.0, 1.0)\tPOLYGON_BEGIN SHELL_BEGIN\n",
        "(2.0, 3.0)\n",
        "(4.0, 5.0)\n",
        "(0.0, 1.0)\tSHELL_END POLYGON_END\n",
    ]
    geoms = [Polygon(shell=[(0, 1), (2, 3), (4, 5), (0, 1)])]
    output = io.StringIO()
    serialize_flat(flatten(geoms), output)
    output.seek(0)

    actual = output.readlines()
    assert actual == expected


def test_serialize_flat_polygon_with_holes():
    expected = [
        "(0.0, 1.0)\tPOLYGON_BEGIN SHELL_BEGIN\n",
        "(2.0, 3.0)\n",
        "(4.0, 5.0)\n",
        "(0.0, 1.0)\tSHELL_END POLYGON_END\n",
        "(0.0, 1.0)\tPOLYGON_BEGIN SHELL_BEGIN\n",
        "(2.0, 3.0)\n",
        "(4.0, 5.0)\n",
        "(0.0, 1.0)\tSHELL_END\n",
        "(6.0, 7.0)\tHOLE_BEGIN\n",
        "(8.0, 9.0)\n",
        "(10.0, 11.0)\n",
        "(6.0, 7.0)\tHOLE_END\n",
        "(12.0, 13.0)\tHOLE_BEGIN\n",
        "(14.0, 15.0)\n",
        "(16.0, 17.0)\n",
        "(12.0, 13.0)\tHOLE_END POLYGON_END\n",
    ]
    geoms = [
        Polygon(shell=((0, 1), (2, 3), (4, 5))),
        Polygon(
            shell=((0, 1), (2, 3), (4, 5)),
            holes=[((6, 7), (8, 9), (10, 11)), ((12, 13), (14, 15), (16, 17))],
        ),
    ]
    output = io.StringIO()
    serialize_flat(flatten(geoms), output)
    output.seek(0)

    actual = output.readlines()
    assert actual == expected


def test_deserialize_flat_points():
    geoms = [Point(0, 1), Point(2, 3, 4)]
    expected_tagged_points = list(flatten(geoms))

    buffer = io.StringIO()
    serialize_flat(expected_tagged_points, buffer)
    buffer.seek(0)

    actual_tagged_points = list(deserialize_flat(buffer))
    assert actual_tagged_points == expected_tagged_points


def test_deserialize_flat_linestring():
    geoms = [LineString([(0, 1), (2, 3), (4, 5)])]
    expected_tagged_points = list(flatten(geoms))

    buffer = io.StringIO()
    serialize_flat(expected_tagged_points, buffer)
    buffer.seek(0)

    actual_tagged_points = list(deserialize_flat(buffer))
    assert actual_tagged_points == expected_tagged_points


def test_deserialize_flat_polygon():
    geoms = [Polygon(shell=[(0, 1), (2, 3), (4, 5), (0, 1)])]
    expected_tagged_points = list(flatten(geoms))

    buffer = io.StringIO()
    serialize_flat(expected_tagged_points, buffer)
    buffer.seek(0)

    actual_tagged_points = list(deserialize_flat(buffer))
    assert actual_tagged_points == expected_tagged_points


def test_deserialize_flat_polygon_with_holes():
    geoms = [
        Polygon(shell=((0, 1), (2, 3), (4, 5))),
        Polygon(
            shell=((0, 1), (2, 3), (4, 5)),
            holes=[((6, 7), (8, 9), (10, 11)), ((12, 13), (14, 15), (16, 17))],
        ),
    ]
    expected_tagged_points = list(flatten(geoms))

    buffer = io.StringIO()
    serialize_flat(expected_tagged_points, buffer)
    buffer.seek(0)

    actual_tagged_points = list(deserialize_flat(buffer))
    assert actual_tagged_points == expected_tagged_points


def test_deserialize_flat_blank_line():
    buffer = io.StringIO("")
    tagged_points = list(deserialize_flat(buffer))
    assert tagged_points == []

    buffer = io.StringIO("\n\t  \n")
    tagged_points = list(deserialize_flat(buffer))
    assert tagged_points == []


def test_deserialize_flat_garbage():
    buffer = io.StringIO("asdf\n")
    tagged_points = list(deserialize_flat(buffer))
    assert tagged_points == []


def test_deserialize_flat_non_tuple():
    buffer = io.StringIO("[0, 1]\n")
    tagged_points = list(deserialize_flat(buffer))
    assert tagged_points == []


def test_deserialize_flat_tuple_numeric():
    buffer = io.StringIO("('a', 'b')\n")
    tagged_points = list(deserialize_flat(buffer))
    assert tagged_points == []


def test_deserialize_flat_tuple_length():
    buffer = io.StringIO("(,)\n")
    tagged_points = list(deserialize_flat(buffer))
    assert tagged_points == []

    buffer = io.StringIO("(1, 2, 3, 4)\n")
    tagged_points = list(deserialize_flat(buffer))
    assert tagged_points == []


def test_deserialize_flat_invalid_tag():
    buffer = io.StringIO("(1, 2)\tASDF LINESTRING_BEGIN\n")
    tagged_points = list(deserialize_flat(buffer))
    assert tagged_points == []


def test_deserialize_flat_invalid_separator():
    buffer = io.StringIO("(1, 2) LINESTRING_BEGIN\n")
    tagged_points = list(deserialize_flat(buffer))
    assert tagged_points == []
