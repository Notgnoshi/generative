import itertools
import logging
from enum import Enum, auto
from typing import Iterable, Tuple, Union

import numpy as np
import shapely.geometry
from more_itertools import peekable
from shapely.geometry import (
    GeometryCollection,
    LineString,
    MultiLineString,
    MultiPoint,
    MultiPolygon,
    Point,
    Polygon,
)
from sklearn.decomposition import PCA, TruncatedSVD
from sklearn.manifold import MDS, TSNE, Isomap, LocallyLinearEmbedding

logger = logging.getLogger(name=__name__)
Geometry = shapely.geometry.base.BaseGeometry


class PointTag(Enum):
    LINESTRING_BEGIN = auto()
    LINESTRING_END = auto()

    POLYGON_BEGIN = auto()
    POLYGON_END = auto()
    SHELL_BEGIN = auto()
    SHELL_END = auto()
    HOLE_BEGIN = auto()
    HOLE_END = auto()

    MULTIPOINT_BEGIN = auto()
    MULTIPOINT_END = auto()

    MULTILINESTRING_BEGIN = auto()
    MULTILINESTRING_END = auto()

    MULTIPOLYGON_BEGIN = auto()
    MULTIPOLYGON_END = auto()

    COLLECTION_BEGIN = auto()
    COLLECTION_END = auto()


TaggedPoint = Tuple[Tuple[float], Union[None, PointTag, Tuple[PointTag]]]
TaggedPointSequence = Iterable[TaggedPoint]


class PointConversion:
    @classmethod
    def to_points(cls, geometries: Iterable[Geometry]) -> TaggedPointSequence:
        """Convert the given geometries to a sequence of tagged points."""
        for geometry in geometries:
            yield from cls._to_points(geometry)

    @classmethod
    def _to_points(cls, geometry: Geometry, recursion_level=0) -> TaggedPointSequence:
        indent = "  " * recursion_level
        logger.debug(indent + f"Converting {geometry.geom_type} to tagged points.")

        if isinstance(geometry, Point):
            yield geometry.coords[0], None
        elif isinstance(geometry, LineString):
            yield from cls._coord_sequence(geometry.coords, PointTag.LINESTRING_BEGIN)
        elif isinstance(geometry, Polygon):
            shell = cls._coord_sequence(geometry.exterior.coords, PointTag.SHELL_BEGIN)
            holes = itertools.chain.from_iterable(
                cls._coord_sequence(h.coords, PointTag.HOLE_BEGIN) for h in geometry.interiors
            )
            points = itertools.chain(shell, holes)
            yield from cls._wrap_sequence(points, PointTag.POLYGON_BEGIN)
        elif isinstance(geometry, MultiPoint):
            points = itertools.chain.from_iterable(
                cls._to_points(g, recursion_level + 1) for g in geometry.geoms
            )
            yield from cls._wrap_sequence(points, PointTag.MULTIPOINT_BEGIN)
        elif isinstance(geometry, MultiLineString):
            points = itertools.chain.from_iterable(
                cls._to_points(g, recursion_level + 1) for g in geometry.geoms
            )
            yield from cls._wrap_sequence(points, PointTag.MULTILINESTRING_BEGIN)
        elif isinstance(geometry, MultiPolygon):
            points = itertools.chain.from_iterable(
                cls._to_points(g, recursion_level + 1) for g in geometry.geoms
            )
            yield from cls._wrap_sequence(points, PointTag.MULTIPOLYGON_BEGIN)
        elif isinstance(geometry, GeometryCollection):
            points = itertools.chain.from_iterable(
                cls._to_points(g, recursion_level + 1) for g in geometry.geoms
            )
            yield from cls._wrap_sequence(points, PointTag.COLLECTION_BEGIN)
        else:
            logger.error(f"Unsupported geometry type '{type(geometry)}'")

    @staticmethod
    def _coord_sequence(coords: Iterable[Tuple[float]], begin_tag: PointTag) -> TaggedPointSequence:
        yield coords[0], begin_tag
        for point in coords[1:-1]:
            yield point, None
        yield coords[-1], PointTag(begin_tag.value + 1)

    @classmethod
    def _wrap_sequence(
        cls, points: TaggedPointSequence, begin_tag: PointTag
    ) -> TaggedPointSequence:
        # Assume there will always be at least two points.
        first_point, first_tag = next(points)
        first_tag = cls._prepend_tag(begin_tag, first_tag)
        yield first_point, first_tag

        last_point, last_tag = next(points)

        for point, tag in points:
            yield last_point, last_tag
            last_point, last_tag = point, tag

        last_tag = cls._append_tag(PointTag(begin_tag.value + 1), last_tag)
        yield last_point, last_tag

    @staticmethod
    def _append_tag(
        tag: PointTag, tags: Union[None, PointTag, Tuple[PointTag]]
    ) -> Union[PointTag, Tuple[PointTag]]:
        if tags is None:
            return tag
        if isinstance(tags, PointTag):
            return (tags, tag)
        if isinstance(tags, tuple):
            return (*tags, tag)
        raise TypeError("Unsupported tags type")

    @staticmethod
    def _prepend_tag(
        tag: PointTag, tags: Union[None, PointTag, Tuple[PointTag]]
    ) -> Union[PointTag, Tuple[PointTag]]:
        if tags is None:
            return tag
        if isinstance(tags, PointTag):
            return (tag, tags)
        if isinstance(tags, tuple):
            return (tag, *tags)
        raise TypeError("Unsupported tags type")

    @classmethod
    def from_points(cls, points: TaggedPointSequence) -> Iterable[Geometry]:
        """Convert the sequence of tagged points back into a sequence of geometries."""
        points = peekable(points)
        while points:
            geometry, _ = cls._get_geometry(points)
            yield geometry

    @classmethod
    def _get_geometry(cls, points: TaggedPointSequence, recursion_level=0) -> Geometry:
        indent = "  " * recursion_level
        point, tags = points.peek()

        first_tag, _ = cls._unwrap_first_tag(tags)
        if (
            first_tag is None
            or first_tag == PointTag.MULTIPOINT_END
            or first_tag == PointTag.COLLECTION_END
        ):
            logger.debug(indent + f"Base case: Point{point}")
            point, tags = next(points)
            return Point(point), tags
        if first_tag in (PointTag.LINESTRING_BEGIN, PointTag.SHELL_BEGIN, PointTag.HOLE_BEGIN):
            logger.debug(indent + f"Base case: {points.peek()}")
            return cls._unwrap_coordinate_sequence(points, recursion_level + 1)

        logger.debug(indent + f"Getting multi-part geometry: {points.peek()}")
        geometry, remaining = cls._get_multipart_geometry(points, recursion_level + 1)
        logger.debug(indent + f"Got multi-part geometry: {geometry.wkt}")
        return geometry, remaining

    @classmethod
    def _get_multipart_geometry(cls, points: TaggedPointSequence, recursion_level=0) -> Geometry:
        indent = "  " * recursion_level
        point, tags = next(points)

        # Unwrap outer tag, and _get_geometry() until we find the matching end tag.
        begin_tag, remaining_tags = cls._unwrap_first_tag(tags)
        end_tag = PointTag(begin_tag.value + 1)
        points.prepend((point, remaining_tags))

        outer_tag = None
        primitives = []
        while outer_tag != end_tag:
            logger.debug(indent + f"Getting primitive: {points.peek()}")
            primitive, remaining_tags = cls._get_geometry(points, recursion_level + 1)
            logger.debug(indent + f"Got primitive {primitive.wkt}")
            primitives.append(primitive)
            outer_tag, remaining_tags = cls._unwrap_first_tag(remaining_tags)

        # Reconstruct the multi-part geometry from the primitives we parsed.
        if begin_tag == PointTag.POLYGON_BEGIN:
            shell = primitives.pop(0)
            holes = primitives
            return Polygon(shell, holes), remaining_tags
        if begin_tag == PointTag.MULTIPOINT_BEGIN:
            return MultiPoint(primitives), remaining_tags
        if begin_tag == PointTag.MULTILINESTRING_BEGIN:
            return MultiLineString(primitives), remaining_tags
        if begin_tag == PointTag.MULTIPOLYGON_BEGIN:
            return MultiPolygon(primitives), remaining_tags
        if begin_tag == PointTag.COLLECTION_BEGIN:
            return GeometryCollection(primitives), remaining_tags

    @staticmethod
    def _unwrap_first_tag(
        tags: Union[PointTag, Tuple[PointTag]]
    ) -> Tuple[PointTag, Union[None, PointTag, Tuple[PointTag]]]:
        """Unwrap the first and any remaining tags."""
        if tags is None:
            return None, None
        if isinstance(tags, PointTag):
            return tags, None
        return tags[0], tags[1:] if len(tags) > 2 else tags[1]

    @classmethod
    def _unwrap_coordinate_sequence(
        cls, points: TaggedPointSequence, recursion_level=0
    ) -> Tuple[Geometry, Union[None, PointTag, Tuple[PointTag]]]:
        indent = "  " * recursion_level
        point, tag = next(points)
        logger.debug(indent + f"Unwrapping CS Point{point}")
        unwrapped = [point]

        point, tag = next(points)
        logger.debug(indent + f"Unwrapping CS Point{point}")
        while tag is None:
            unwrapped.append(point)
            point, tag = next(points)
            logger.debug(indent + f"Unwrapping CS Point{point}")
        unwrapped.append(point)

        if isinstance(tag, PointTag):
            remaining_tags = None
        if isinstance(tag, tuple):
            remaining_tags = tag[1:] if len(tag) > 2 else tag[1]

        return LineString(unwrapped), remaining_tags


def project(geometries: Iterable[Geometry], kind="pca") -> Iterable[Geometry]:
    """Project the given geometries to 2D.

    :param kind: The type of projection to use. Can be one of 'pca', 'svd', 'isometric', 'isometric-auto', 'xy', 'xz', or 'yz'.
    """
    tagged_point_sequence = PointConversion.to_points(geometries)
    if kind in ("xy", "xz", "yz"):
        transformed_point_sequence = _drop_coord(tagged_point_sequence, basis=kind)
    elif kind in ("pca", "svd", "mds", "isomap", "tsne", "lle", "isometric", "isometric-auto"):
        transformed_point_sequence = _fit_transform(tagged_point_sequence, kind=kind)
    # Really only useful for pretending projection works while working on it.
    elif kind == "I":
        transformed_point_sequence = tagged_point_sequence
    else:
        raise ValueError(f"Unsupported projection type '{kind=}'")

    return PointConversion.from_points(transformed_point_sequence)


def unzip(iterable):
    return zip(*iterable)


def _fit_transform(tagged_points: TaggedPointSequence, kind) -> TaggedPointSequence:
    """Perform PCA on the given geometries."""
    points, tags = unzip(tagged_points)

    # Convert the generator of points to an array of points.
    # This will consume the generator, and keep the points loaded in memory.
    points = np.array(list(_zeropad_3d(points)))

    points *= 10

    # TruncatedSVD picked a sideways view
    # PCA picked a top-down view
    if kind == "pca":
        decomp = PCA(n_components=2)
    elif kind == "svd":
        decomp = TruncatedSVD(n_components=2, n_iter=5)
    elif kind == "mds":
        decomp = MDS(n_components=2)
    elif kind == "tsne":
        decomp = TSNE(n_components=2, perplexity=30)
    elif kind == "isomap":
        decomp = Isomap(n_components=2, n_neighbors=5)
    elif kind == "lle":
        decomp = LocallyLinearEmbedding(n_components=2, n_neighbors=5, eigen_solver="dense")
    else:
        raise ValueError(f"Unsupported projection '{kind}'")
    transformed = decomp.fit_transform(points)

    for point, tag in zip(transformed, tags):
        yield point, tag


def _zeropad_3d(points: Iterable[Tuple[float]]) -> Iterable[Tuple[float]]:
    padding = (0, 0, 0)
    return ((*point, *padding)[:3] for point in points)


def _drop_coord(tagged_points: TaggedPointSequence, basis: str) -> TaggedPointSequence:
    """Project the given 3D geometry objects onto one of the standard 2D bases."""
    # Do not allow flips. That is, you cannot reorder coordinates, only drop.
    if basis == "xy":
        coord = 2
    elif basis == "xz":
        coord = 1
    elif basis == "yz":
        coord = 0
    else:
        raise ValueError(f"Unsupported basis for dropping coordinates '{basis=}'")
    points, tags = unzip(tagged_points)
    for point, tag in zip(_zeropad_3d(points), tags):
        yield (*point[:coord], *point[coord + 1 :]), tag
