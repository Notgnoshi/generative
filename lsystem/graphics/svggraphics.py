import svgwrite

from .graphics import GraphicsWriter


class SvgWriter(GraphicsWriter):
    """An L-Systems renderer for SVG drawings.

    TODO: Use an R-Tree or KD-Tree to assist in simplifying overlapping lines.
    """
