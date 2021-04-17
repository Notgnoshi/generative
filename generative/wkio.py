import ast
import io
import logging
from typing import Iterable

import shapely.geometry
from shapely import wkb, wkt

from generative.flatten import PointTag, TaggedPointSequence, flatten, unflatten

Geometry = shapely.geometry.base.BaseGeometry
logger = logging.getLogger(name=__name__)


def _parse_wkt(buffer: io.TextIOWrapper) -> Iterable[Geometry]:
    for line in buffer:
        line = line.strip()
        try:
            geometry = wkt.loads(line)
            logger.debug(f"loaded {geometry}")
            yield geometry
        except shapely.errors.WKTReadingError:
            logger.error(f"Failed to parse {line=}")


def _parse_wkb(buffer: io.TextIOWrapper) -> Iterable[Geometry]:
    for line in buffer:
        line = line.strip()
        try:
            geometry = wkb.loads(line, hex=True)
            logger.debug(f"loaded {geometry}")
            yield geometry
        except shapely.errors.WKBReadingError:
            logger.error(f"Failed to parse {line=}")


def deserialize_flat(buffer: io.TextIOWrapper) -> TaggedPointSequence:
    r"""Deserialize a flattened sequence of points.

    Sample Input/Output:
        (0, 0)\n
            POINT(0 0)

        (0, 0)\tLINESTRING_BEGIN\n
        (1, 1)\n
        (2, 2)\tLINESTRING_END\n
            LINESTRING(0 0, 1 1, 2 2):

        (0, 1)\tPOLYGON_BEGIN SHELL_BEGIN\n
        (2, 3)\n
        (4, 5)\n
        (0, 1)\tSHELL_END POLYGON_END\n
            POLYGON((0 1, 2 3, 4 5)):

        (0, 1)\tPOLYGON_BEGIN SHELL_BEGIN\n
        (2, 3)\n
        (4, 5)\n
        (0, 1)\tSHELL_END POLYGON_END\n
        (0, 1)\tPOLYGON_BEGIN SHELL_BEGIN\n
        (2, 3)\n
        (4, 5)\n
        (0, 1)\tSHELL_END\n
        (6, 7)\tHOLE_BEGIN\n
        (8, 9)\n
        (10, 11)\n
        (6, 7)\tHOLE_END\n
        (12, 13)\tHOLE_BEGIN\n
        (14, 15)\n
        (16, 17)\n
        (12, 13)\tHOLE_END POLYGON_END\n
            POLYGON((0 1, 2 3, 4 5), POLYGON((0 1, 2 3, 4 5), (6 7, 8 9, 10 11), (12 13, 14 15, 16 17)):

    Example:
        >>> geoms = [Point(0, 0), LineString([(1, 1), (2, 2), (3, 3)])]
        >>> tagged_points = list(flatten(geoms))
        >>> buffer = io.StringIO()
        >>> serialize_flat(tagged_points, buffer)
        >>> # Reset cursor to beginning
        >>> buffer.seek(0)
        >>> new_tagged_points = list(deserialize_flat(buffer))
        >>> assert new_tagged_points == tagged_points
    """
    for line in buffer:
        line = line.strip()
        parts = line.split("\t")
        tags = tuple()
        if not parts:
            # Blank line
            continue

        # Fuck this.
        point = parts[0]
        try:
            # Parse the tuple of floats.
            point = ast.literal_eval(point)
        except BaseException as e:
            logger.warning("Could not interpret '%s' as a tuple. Ignoring...", point, exc_info=e)
            continue

        if not isinstance(point, tuple):
            logger.warning("Did not interpret '%s' as a tuple. Ignoring...", point)
            continue
        if not all(isinstance(c, (int, float)) for c in point):
            logger.warning("Point '%s' must be numeric. Ignoring...", point)
            continue
        if len(point) not in (2, 3):
            logger.warning("Point '%s' must be 2D or 3D. Ignoring...", point)
            continue

        if len(parts) > 1:
            tags = parts[1:]
            tags = " ".join(tags)
            tags = tags.split()
            try:
                # Convert the Enum name to the enumeration.
                tags = tuple(PointTag[tag] for tag in tags)
            except BaseException as e:
                logger.warning("Failed to parse tags '%s'. Ignoring...", tags)
                continue
        yield point, tags


def deserialize_geometries(buffer: io.TextIOWrapper, fmt="wkt") -> Iterable[Geometry]:
    if fmt == "wkt":
        return _parse_wkt(buffer)
    if fmt == "wkb":
        return _parse_wkb(buffer)
    # If you're using flattened geometries, you probably want to act on them as a point cloud,
    # not a sequence of geometries. But this deserialization method is provided regardless.
    if fmt == "flat":
        return unflatten(deserialize_flat(buffer))
    raise ValueError(f"{fmt=} unsupported")


def _serialize_wkt(geometries: Iterable[Geometry], buffer: io.TextIOWrapper):
    # TODO: Determine how much to chunk before writing.
    for geometry in geometries:
        wkt.dump(geometry, buffer, trim=True)
        buffer.write("\n")


def _serialize_wkb(geometries: Iterable[Geometry], buffer: io.TextIOWrapper):
    # TODO: Determine how much to chunk before writing.
    for geometry in geometries:
        wkb.dump(geometry, buffer, hex=True)
        buffer.write("\n")


def serialize_flat(tagged_points: TaggedPointSequence, buffer: io.TextIOWrapper):
    r"""Serialize the given flattened geometries in a custom format.

    Sample Input/Output:
        POINT(0 0):
            (0, 0)\n

        LINESTRING(0 0, 1 1, 2 2):
            (0, 0)\tLINESTRING_BEGIN\n
            (1, 1)\n
            (2, 2)\tLINESTRING_END\n

        POLYGON((0 1, 2 3, 4 5)):
            (0, 1)\tPOLYGON_BEGIN SHELL_BEGIN\n
            (2, 3)\n
            (4, 5)\n
            (0, 1)\tSHELL_END POLYGON_END\n

        POLYGON((0 1, 2 3, 4 5), POLYGON((0 1, 2 3, 4 5), (6 7, 8 9, 10 11), (12 13, 14 15, 16 17)):
            (0, 1)\tPOLYGON_BEGIN SHELL_BEGIN\n
            (2, 3)\n
            (4, 5)\n
            (0, 1)\tSHELL_END POLYGON_END\n
            (0, 1)\tPOLYGON_BEGIN SHELL_BEGIN\n
            (2, 3)\n
            (4, 5)\n
            (0, 1)\tSHELL_END\n
            (6, 7)\tHOLE_BEGIN\n
            (8, 9)\n
            (10, 11)\n
            (6, 7)\tHOLE_END\n
            (12, 13)\tHOLE_BEGIN\n
            (14, 15)\n
            (16, 17)\n
            (12, 13)\tHOLE_END POLYGON_END\n

    The format is designed to allow acting on the geometries as if they were a point cloud, while
    maintaining individual geometries. This allows for, e.g., using PCA to project 3D geometries
    to 2D.

    Note that most operations consuming this format will be unable to operate on it as a stream.
    They'll likely have to collapse the input stream in memory, and act on it all at once.

    Example:
        >>> geoms = [Point(0, 0), LineString([(1, 1), (2, 2), (3, 3)])]
        >>> tagged_points = list(flatten(geoms))
        >>> buffer = io.StringIO()
        >>> serialize_flat(tagged_points, buffer)
        >>> # Reset cursor to beginning
        >>> buffer.seek(0)
        >>> new_tagged_points = list(deserialize_flat(buffer))
        >>> assert new_tagged_points == tagged_points
    """
    for point, tags in tagged_points:
        buffer.write(str(point))
        if tags:
            buffer.write("\t" + " ".join(tag.name for tag in tags))
        buffer.write("\n")


def serialize_geometries(geometries: Iterable[Geometry], buffer: io.TextIOWrapper, fmt="wkt"):
    if fmt == "wkt":
        _serialize_wkt(geometries, buffer)
    elif fmt == "wkb":
        _serialize_wkb(geometries, buffer)
    elif fmt == "flat":
        serialize_flat(flatten(geometries), buffer)
    else:
        raise ValueError(f"{fmt=} unsupported")
