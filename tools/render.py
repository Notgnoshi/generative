#!/usr/bin/env python3
"""Render 3D Lindenmayer Systems in an interactive OpenGL window."""
import argparse
import itertools
import logging
import pathlib
import sys
from typing import Iterable

import numpy as np
import shapely.geometry
import vispy.scene
from shapely.geometry import (
    GeometryCollection,
    LineString,
    MultiLineString,
    MultiPoint,
    MultiPolygon,
    Point,
    Polygon,
)

root = pathlib.Path(__file__).resolve().parent.parent
sys.path.insert(0, str(root))
from generative.wkio import deserialize_geometries

LOG_LEVELS = {
    "CRITICAL": logging.CRITICAL,
    "ERROR": logging.ERROR,
    "WARNING": logging.WARNING,
    "INFO": logging.INFO,
    "DEBUG": logging.DEBUG,
}
DEFAULT_LEVEL = "INFO"
Geometry = shapely.geometry.base.BaseGeometry


def pairwise(iterable: Iterable) -> Iterable:
    """Iterate over the given iterable in pairs.

    Example:
        s -> (s0,s1), (s1,s2), (s2, s3), ...
    """
    a, b = itertools.tee(iterable)
    next(b, None)
    return zip(a, b)


def parse_args():
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )

    parser.add_argument(
        "--input",
        "-i",
        type=argparse.FileType("r"),
        default=sys.stdin,
        help="The WKT/WKB input to parse. Defaults to stdin.",
    )
    parser.add_argument(
        "--axis", action="store_true", default=False, help="Display an XYZ axis on screen"
    )

    parser.add_argument(
        "--input-format",
        "-I",
        default="wkt",
        choices=["wkt", "wkb", "flat"],
        help="The input geometry format. Defaults to WKT.",
    )
    parser.add_argument(
        "-l",
        "--log-level",
        type=str,
        default=DEFAULT_LEVEL,
        choices=LOG_LEVELS.keys(),
        help=f"Set the logging output level. Defaults to {DEFAULT_LEVEL}.",
    )

    return parser.parse_args()


def ensure_3d(points):
    for point in points:
        template = [0, 0, 0]
        template[: len(point)] = point
        yield template


def add_cs(coords, segments):
    for pair in pairwise(coords):
        segments.append(tuple(ensure_3d(pair)))


def add_c(coords, points):
    for coord in ensure_3d(coords):
        points.append(coord)


def add(geometry: Geometry, points, segments):
    if isinstance(geometry, (MultiPolygon, MultiLineString, MultiPoint, GeometryCollection)):
        for geom in geometry.geoms:
            add(geom, points, segments)
    elif isinstance(geometry, Point):
        add_c(geometry.coords, points)
    elif isinstance(geometry, Polygon):
        add_cs(geometry.exterior.coords, segments)
    else:
        add_cs(geometry.coords, segments)


def main(args):
    logger.debug(args)
    geometries = deserialize_geometries(args.input, args.input_format)

    canvas = vispy.scene.SceneCanvas(title=__file__, keys="interactive")
    # TODO: After the points are loaded, use PCA to pick an appropriate default camera view?
    # TODO: Perhaps default to an isometric view?
    # TODO: Add an argument to allow a constantly spinning turntable axis.
    view = canvas.central_widget.add_view(camera="arcball")

    points = []
    # Use vanilla Python list for segments, because it's optimized for repeated appends.
    # Will be reshaped later into a contiguous numpy array of points.
    # Each segment is a 2-tuple of 3D coordinates.
    segments = []

    logger.debug("Loading geometries...")
    # TODO: Examine performance.
    for geometry in geometries:
        add(geometry, points, segments)
    logger.info("Loaded %d segments and %d points.", len(segments), len(points))

    segments = np.array(segments)
    segments = segments.reshape((-1, 3))
    points = np.array(points)
    points = points.reshape((-1, 3))
    logger.debug("segments: %s", segments)
    logger.debug("points: %s", points)

    logger.debug("Adding geometries to scene...")
    if len(points) != 0:
        markers = vispy.scene.visuals.Markers(parent=view.scene)
        markers.set_data(
            points,
            edge_color=None,
            edge_width=0,
            edge_width_rel=None,
            face_color=(0.8, 0.8, 0.8, 0.4),
            size=8,
            scaling=False,
        )

    if len(segments) != 0:
        vispy.scene.visuals.Line(
            segments, color=(0.8, 0.8, 0.8, 0.4), connect="segments", parent=view.scene
        )

    if args.axis:
        # TODO: Figure out how to scale the axis up?
        # TODO: Decide if I actually want the axis pinned to a corner of the viewbox.
        # TODO: Put the axis at the geometries' centroid.
        vispy.scene.visuals.XYZAxis(parent=view.scene)
    logger.debug("Added geometries to scene.")

    # Auto-scale
    view.camera.set_range()
    canvas.show()
    try:
        # TODO: Add a button that, when clicked, outputs the geometries using the current camera's
        # perspective as the 3D -> 2D projection.
        vispy.app.run()
    except KeyboardInterrupt:
        vispy.app.quit()


if __name__ == "__main__":
    args = parse_args()
    logging.basicConfig(
        format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
        level=LOG_LEVELS.get(args.log_level),
        stream=sys.stderr,
    )
    logger = logging.getLogger(name=__file__)

    main(args)
