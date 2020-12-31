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
from generative.wkio import deserialize_geometries

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
