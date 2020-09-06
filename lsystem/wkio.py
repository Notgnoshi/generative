import io
import logging
import sys
from typing import Iterable

import shapely.geometry
from shapely import wkb, wkt

Geometry = shapely.geometry.base.BaseGeometry
logging.basicConfig(
    format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
    stream=sys.stderr,
)
logger = logging.getLogger(name=__name__)


def _parse_wkt(buffer: io.TextIOWrapper) -> Iterable[Geometry]:
    for line in buffer.readlines():
        line = line.strip()
        try:
            geometry = wkt.loads(line)
            logger.debug(f"loaded {geometry}")
            yield geometry
        except shapely.errors.WKTReadingError:
            logger.error(f"Failed to parse {line=}")


def _parse_wkb(buffer: io.TextIOWrapper) -> Iterable[Geometry]:
    for line in buffer.readlines():
        line = line.strip()
        try:
            geometry = wkb.loads(line, hex=True)
            logger.debug(f"loaded {geometry}")
            yield geometry
        except shapely.errors.WKBReadingError:
            logger.error(f"Failed to parse {line=}")


def deserialize_geometries(buffer: io.TextIOWrapper, fmt="wkt") -> Iterable[Geometry]:
    if fmt == "wkt":
        return _parse_wkt(buffer)
    if fmt == "wkb":
        return _parse_wkb(buffer)
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


def serialize_geometries(geometries: Iterable[Geometry], buffer: io.TextIOWrapper, fmt="wkt"):
    if fmt == "wkt":
        _serialize_wkt(geometries, buffer)
    elif fmt == "wkb":
        _serialize_wkb(geometries, buffer)
    else:
        raise ValueError(f"{fmt=} unsupported")
