import itertools
import logging
from enum import Enum, auto
from math import radians
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

logger = logging.getLogger(name=__name__)
Geometry = shapely.geometry.base.BaseGeometry


class PointTag(Enum):
    """Tags to mark points in a coordinate sequence.

    Note that each _END tag _must_ be the corresponding _BEGIN tag +1.

    Further, note that there is no tag for POINTs, because a point is any coordinate that's not
    wrapped between two _BEGIN and _END tags.
    """

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


Tag = Tuple[PointTag]
TaggedPoint = Tuple[Tuple[float], Tag]
TaggedPointSequence = Iterable[TaggedPoint]


def flatten(geometries: Iterable[Geometry]) -> TaggedPointSequence:
    """Convert the given geometries to a sequence of tagged points."""
    for geometry in geometries:
        yield from flatten_single(geometry)


def flatten_single(geometry: Geometry, recursion_level=0) -> TaggedPointSequence:
    """Recursively convert a single geometry to a sequence of tagged points."""
    indent = "  " * recursion_level
    logger.debug(indent + "Converting %s to tagged points.", geometry.geom_type)

    if isinstance(geometry, Point):
        yield geometry.coords[0], ()
    elif isinstance(geometry, LineString):
        yield from wrap_bare(geometry.coords, PointTag.LINESTRING_BEGIN)
    elif isinstance(geometry, Polygon):

        shell = wrap_bare(geometry.exterior.coords, PointTag.SHELL_BEGIN)
        holes = itertools.chain.from_iterable(
            wrap_bare(h.coords, PointTag.HOLE_BEGIN) for h in geometry.interiors
        )
        points = itertools.chain(shell, holes)
        yield from wrap_tagged(points, PointTag.POLYGON_BEGIN)
    elif isinstance(geometry, MultiPoint):
        points = itertools.chain.from_iterable(
            flatten_single(g, recursion_level + 1) for g in geometry.geoms
        )
        yield from wrap_tagged(points, PointTag.MULTIPOINT_BEGIN)
    elif isinstance(geometry, MultiLineString):
        points = itertools.chain.from_iterable(
            flatten_single(g, recursion_level + 1) for g in geometry.geoms
        )
        yield from wrap_tagged(points, PointTag.MULTILINESTRING_BEGIN)
    elif isinstance(geometry, MultiPolygon):
        points = itertools.chain.from_iterable(
            flatten_single(g, recursion_level + 1) for g in geometry.geoms
        )
        yield from wrap_tagged(points, PointTag.MULTIPOLYGON_BEGIN)
    elif isinstance(geometry, GeometryCollection):
        points = itertools.chain.from_iterable(
            flatten_single(g, recursion_level + 1) for g in geometry.geoms
        )
        yield from wrap_tagged(points, PointTag.COLLECTION_BEGIN)
    else:
        logger.error(indent + "Unsupported geometry type '%s'", type(geometry))


def wrap_bare(coords: Iterable[Tuple[float]], begin_tag: PointTag) -> TaggedPointSequence:
    """Wrap the given coordinate seqence in the given tag type.

    You pass in the _BEGIN tag, with the assumption that the _END tag is _BEGIN+1.
    """
    yield coords[0], (begin_tag,)
    for point in coords[1:-1]:
        yield point, ()
    yield coords[-1], (PointTag(begin_tag.value + 1),)


def wrap_tagged(points: TaggedPointSequence, begin_tag: PointTag) -> TaggedPointSequence:
    """Wrap an already tagged point sequence in another layer of _BEGIN and _END tags."""
    # Assume there will always be at least two points.
    first_point, first_tag = next(points)
    first_tag = (begin_tag,) + first_tag
    yield first_point, first_tag

    last_point, last_tag = next(points)

    for point, tag in points:
        yield last_point, last_tag
        last_point, last_tag = point, tag

    last_tag = last_tag + (PointTag(begin_tag.value + 1),)
    yield last_point, last_tag


def unflatten(points: TaggedPointSequence) -> Iterable[Geometry]:
    """Convert the sequence of tagged points back into a sequence of geometries."""
    # We need to be able to peek at the next point in the sequence without consuming it.
    points = peekable(points)
    while points:
        geometry, _ = unflatten_single(points)
        yield geometry


def unflatten_single(points: TaggedPointSequence, recursion_level=0) -> Geometry:
    """Get the next geometry from the given sequence of tagged points.

    All nested geometries will be reconstructed and returned.
    In order to nicely handle the recursive cases, this method, along with __unflatten_multipart(),
    and __unwrap_coordinate_sequence() take in their recursion level to facilitate nicer looking
    logging, and return any remaining tags left to unwrap.

    These remaining tags only occur when unflattening a multipart geometry. That is, a MULTI*
    geometry, a POLYGON, or a GEOMETRYCOLLECTION, but note that __unflatten_multipart() internally
    calls unflatten_single() to get each component of a multipart geometry.
    """
    indent = "  " * recursion_level
    point, tags = points.peek()

    first_tag, _ = __unwrap_first_tag(tags)
    if (
        not first_tag
        or first_tag == PointTag.MULTIPOINT_END
        or first_tag == PointTag.COLLECTION_END
    ):
        logger.debug(indent + "Base case: Point%s", point)
        point, tags = next(points)
        return Point(point), tags
    if first_tag in (PointTag.LINESTRING_BEGIN, PointTag.SHELL_BEGIN, PointTag.HOLE_BEGIN):
        logger.debug(indent + "Base case: %s", points.peek())
        return __unflatten_coordinate_sequence(points, recursion_level + 1)

    logger.debug(indent + "Getting multi-part geometry: %s", points.peek())
    geometry, remaining = __unflatten_multipart(points, recursion_level + 1)
    logger.debug(indent + "Got multi-part geometry: %s", geometry.wkt)
    return geometry, remaining


def __unflatten_multipart(points: TaggedPointSequence, recursion_level=0) -> Geometry:
    """Recursively handle the multipart case for unflatten_single().

    The multipart case is separate because it requires recursion to unwrap nested tags that result
    from multipart geometries (where a single point could be the begin to multiple geometries, or
    and end to multiple).
    """
    indent = "  " * recursion_level
    point, tags = next(points)

    # Unwrap outer tag, and _get_geometry() until we find the matching end tag.
    begin_tag, remaining_tags = __unwrap_first_tag(tags)
    end_tag = PointTag(begin_tag.value + 1)
    points.prepend((point, remaining_tags))

    outer_tag = None
    primitives = []
    while outer_tag != end_tag:
        logger.debug(indent + "Getting primitive: %s", points.peek())
        primitive, remaining_tags = unflatten_single(points, recursion_level + 1)
        logger.debug(indent + "Got primitive %s", primitive.wkt)
        primitives.append(primitive)
        outer_tag, remaining_tags = __unwrap_first_tag(remaining_tags)

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


def __unwrap_first_tag(tags: Tag) -> Tuple[PointTag, Tag]:
    """Unwrap the first and any remaining tags."""
    if not tags:
        return (), ()
    return tags[0], tags[1:]


def __unflatten_coordinate_sequence(
    points: TaggedPointSequence, recursion_level=0
) -> Tuple[Geometry, Tag]:
    """Unwrap a coordinate sequence as a LINESTRING.

    This can also be used to unwrap polygon shells and holes.
    TODO: There has _got_ to be a better implementation than this.
    """
    indent = "  " * recursion_level
    point, tag = next(points)
    logger.debug(indent + "Unwrapping CS Point%s", point)
    unwrapped = [point]

    point, tag = next(points)
    logger.debug(indent + "Unwrapping CS Point%s", point)
    while not tag:
        unwrapped.append(point)
        point, tag = next(points)
    logger.debug(indent + "Unwrapping CS Point%s", point)
    unwrapped.append(point)

    _, remaining_tags = __unwrap_first_tag(tag)
    return LineString(unwrapped), remaining_tags


def project(tagged_points: TaggedPointSequence, kind="pca", dimensions=2) -> TaggedPointSequence:
    """Project the given geometries to 2D.

    :param kind: The type of projection to use. Can be one of 'pca', 'svd', 'isometric', 'auto', 'xy', 'xz', or 'yz'.
    :param dimensions: The target dimensionality of the projection for PCA, SVD, or isometric.
    """
    if kind in ("xy", "xz", "yz"):
        transformed_point_sequence = _drop_coord(tagged_points, basis=kind)
    elif kind in ("pca", "svd"):
        transformed_point_sequence = _fit_transform(tagged_points, kind, dimensions)
    elif kind == "isometric":
        transformed_point_sequence = _isometric(tagged_points, dimensions)
    elif kind == "auto":
        # PCA has tended to flip things upside down, to flip about the x axis by 180 and rotate a
        # a bit to ensure no symmetry
        decomp = PCA(n_components=3)
        points, tags = unzip(tagged_points)
        points = np.array(list(_zeropad_3d(points)))
        transformed = decomp.fit_transform(points)
        logger.error(transformed.shape)
        rotation = _rot_x(radians(180)) @ _rot_z(radians(13))
        transformed = transformed @ rotation
        return zip(transformed[:, :dimensions], tags)
    # Really only useful for pretending projection works while working on it.
    elif kind == "I":
        transformed_point_sequence = tagged_points
    else:
        raise ValueError(f"Unsupported projection type '{kind=}'")

    return transformed_point_sequence


def unzip(iterable):
    return zip(*iterable)


def _fit_transform(tagged_points: TaggedPointSequence, kind, dimensions) -> TaggedPointSequence:
    """Project the given geometries."""
    points, tags = unzip(tagged_points)

    # Convert the generator of points to an array of points.
    # This will consume the generator, and keep the points loaded in memory.
    points = np.array(list(_zeropad_3d(points)))

    # TruncatedSVD picked a sideways view
    # PCA picked a top-down view
    if kind == "pca":
        decomp = PCA(n_components=dimensions)
    elif kind == "svd":
        if dimensions >= 3:
            raise ValueError("SVD cannot be used for 3D -> 3D projections")
        decomp = TruncatedSVD(n_components=dimensions, n_iter=5)
    else:
        raise ValueError(f"Unsupported projection '{kind}'")
    transformed = decomp.fit_transform(points)

    return zip(transformed, tags)


def _rot_x(theta):
    """X axis rotation matrix."""
    return np.array(
        [
            [1, 0, 0],
            [0, np.cos(theta), -np.sin(theta)],
            [0, np.sin(theta), np.cos(theta)],
        ]
    )


def _rot_y(theta):
    """Y axis rotation matrix."""
    return np.array(
        [
            [np.cos(theta), 0, np.sin(theta)],
            [0, 1, 0],
            [-np.sin(theta), 0, np.cos(theta)],
        ]
    )


def _rot_z(theta):
    """Z axis rotation matrix."""
    return np.array(
        [
            [np.cos(theta), -np.sin(theta), 0],
            [np.sin(theta), np.cos(theta), 0],
            [0, 0, 1],
        ]
    )


def _isometric(tagged_points: TaggedPointSequence, dimensions) -> TaggedPointSequence:
    """Perform an isometric projection with rotation matrices."""
    # TODO: This isometric projection hasn't given very good results so far. It needs more work.
    rotation = _rot_x(radians(35.264)) @ _rot_y(radians(45))
    points, tags = unzip(tagged_points)
    for point, tag in zip(_zeropad_3d(points), tags):
        yield (np.array(point) @ rotation)[:dimensions], tag


def _zeropad_3d(points: Iterable[Tuple[float]]) -> Iterable[Tuple[float]]:
    padding = (0, 0, 0)
    # NOTE: Setting the interpreter --stepsize to 10 is equivalent.
    # points = (tuple(10 * c for c in point) for point in points)
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
