#!/usr/bin/env python3
"""Convert 2D WKT/WKB to SVG.

3D WKT/WKB input will have the Z coordinate stripped.
"""
import argparse
import logging
import pathlib
import sys

import shapely.geometry
import svgwrite

root = pathlib.Path(__file__).resolve().parent.parent
sys.path.insert(0, str(root))
from generative.wkio import deserialize_geometries  # isort:skip

LOG_LEVELS = {
    "CRITICAL": logging.CRITICAL,
    "ERROR": logging.ERROR,
    "WARNING": logging.WARNING,
    "INFO": logging.INFO,
    "DEBUG": logging.DEBUG,
}
DEFAULT_LEVEL = "WARNING"


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
        "--output",
        "-o",
        type=argparse.FileType("w"),
        default=sys.stdout,
        help="Where to output the SVG. Defaults to stdout.",
    )
    parser.add_argument(
        "--format",
        "-f",
        default="wkt",
        choices=["wkt", "wkb"],
        help="The input format. Defaults to WKT.",
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


def insert_point(
    dwg: svgwrite.Drawing, geom: shapely.geometry.Point, group: svgwrite.container.Group = None
):
    logger.debug("Adding %s", geom)
    p = svgwrite.shapes.Circle(center=geom.coords[0][:2], r=0.5)
    if group is not None:
        group.add(p)
    else:
        dwg.add(p)


def insert_linestring(
    dwg: svgwrite.Drawing, geom: shapely.geometry.LineString, group: svgwrite.container.Group = None
):
    logger.debug("Adding %s", geom)
    l = svgwrite.shapes.Polyline(points=[c[:2] for c in geom.coords])
    if group is not None:
        group.add(l)
    else:
        dwg.add(l)


def insert_polygon(
    dwg: svgwrite.Drawing, geom: shapely.geometry.Polygon, group: svgwrite.container.Group = None
):
    logger.debug("Adding %s", geom)
    p = svgwrite.shapes.Polygon(points=[c[:2] for c in geom.exterior.coords])
    if group is not None:
        group.add(p)
    else:
        dwg.add(p)


def insert_collection(
    dwg: svgwrite.Drawing,
    geoms: shapely.geometry.base.BaseMultipartGeometry,
    group: svgwrite.container.Group = None,
):
    logger.debug("Entering group for %s", geoms.geom_type)
    g = svgwrite.container.group()
    for geom in geoms.geoms:
        insert_geometry(dwg, geom, g)
    logger.debug("Ending group for %s", geoms.geom_type)
    if group is not None:
        group.add(g)
    else:
        dwg.add(g)


inserters = {
    "Point": insert_point,
    "LineString": insert_linestring,
    "LinearRing": insert_linestring,
    "Polygon": insert_polygon,
    "GeometryCollection": insert_collection,
    "MultiPoint": insert_collection,
    "MultiLineString": insert_collection,
    "MultiPolygon": insert_collection,
}


def insert_geometry(
    dwg: svgwrite.Drawing,
    geom: shapely.geometry.base.BaseGeometry,
    group: svgwrite.container.Group = None,
):
    t = geom.geom_type
    inserters[t](dwg, geom, group)


def main(args):
    min_x = 0
    max_x = 0
    min_y = 0
    max_y = 0

    dwg = svgwrite.Drawing()
    dwg.fill(opacity=0.0)
    # NOTE: The default stepsize is also 1, which means it's as long as the lines are wide.
    dwg.stroke(color="black", width=1)

    # TODO: Flip the y-axis because screen coordinates.
    for geom in deserialize_geometries(args.input, args.format):
        mx, my, Mx, My = geom.bounds
        min_x = min(mx, min_x)
        min_y = min(my, min_y)
        max_x = max(Mx, max_x)
        max_y = max(My, max_y)

        insert_geometry(dwg, geom)

    width = max_x - min_x
    height = max_y - min_y
    # The viewbox is in user space (unitless).
    # Any size, if specified, has to be set in the Drawing initializer.
    dwg.viewbox(min_x, min_y, width, height)
    args.output.write(dwg.tostring() + "\n")


if __name__ == "__main__":
    args = parse_args()
    logging.basicConfig(
        format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
        level=LOG_LEVELS.get(args.log_level),
        stream=sys.stderr,
    )
    logger = logging.getLogger(name=__file__)

    main(args)
