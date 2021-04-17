import logging
from math import radians
from typing import Iterable, Tuple

import numpy as np
from sklearn.decomposition import PCA, TruncatedSVD

from generative.flatten import TaggedPointSequence

logger = logging.getLogger(name=__name__)


def project(
    tagged_points: TaggedPointSequence, kind="pca", dimensions=2, scale=1.0
) -> TaggedPointSequence:
    """Project the given geometries to 2D.

    :param kind: The type of projection to use. Can be one of 'I', 'pca', 'svd', 'isometric', 'auto', 'xy', 'xz', or 'yz'.
    :param dimensions: The target dimensionality of the projection for PCA, SVD, or isometric.
    :param scale: A multiplicative scale factor.
    """
    if kind in ("xy", "xz", "yz"):
        transformed_point_sequence = _drop_coord(tagged_points, kind, scale)
    elif kind in ("pca", "svd"):
        transformed_point_sequence = _fit_transform(tagged_points, kind, dimensions, scale)
    elif kind == "isometric":
        transformed_point_sequence = _isometric(tagged_points, dimensions, scale)
    elif kind == "auto":
        # PCA has tended to flip things upside down, to flip about the x axis by 180 and rotate a
        # a bit to ensure no symmetry
        decomp = PCA(n_components=3)
        points, tags = unzip(tagged_points)
        points = scale * np.array(list(_zeropad_3d(points)))
        transformed = decomp.fit_transform(points)
        logger.error(transformed.shape)
        rotation = _rot_x(radians(180)) @ _rot_z(radians(13))
        transformed = transformed @ rotation
        return zip(transformed[:, :dimensions], tags)
    elif kind == "I":
        points, tags = unzip(tagged_points)
        if scale != 1.0:
            points = (tuple(scale * c for c in point) for point in points)
        transformed_point_sequence = zip(points, tags)
    else:
        raise ValueError(f"Unsupported projection type '{kind=}'")

    return transformed_point_sequence


def unzip(iterable):
    return zip(*iterable)


def _fit_transform(
    tagged_points: TaggedPointSequence, kind, dimensions, scale
) -> TaggedPointSequence:
    """Project the given geometries."""
    points, tags = unzip(tagged_points)

    # Convert the generator of points to an array of points.
    # This will consume the generator, and keep the points loaded in memory.
    points = scale * np.array(list(_zeropad_3d(points)))

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


def _isometric(tagged_points: TaggedPointSequence, dimensions, scale) -> TaggedPointSequence:
    """Perform an isometric projection with rotation matrices."""
    # TODO: This isometric projection hasn't given very good results so far. It needs more work.
    rotation = _rot_x(radians(35.264)) @ _rot_y(radians(45))
    points, tags = unzip(tagged_points)
    for point, tag in zip(_zeropad_3d(points), tags):
        yield (scale * np.array(point) @ rotation)[:dimensions], tag


def _zeropad_3d(points: Iterable[Tuple[float]]) -> Iterable[Tuple[float]]:
    padding = (0, 0, 0)
    return ((*point, *padding)[:3] for point in points)


def _drop_coord(tagged_points: TaggedPointSequence, basis: str, scale) -> TaggedPointSequence:
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
        point = (*point[:coord], *point[coord + 1 :])
        point = tuple(scale * c for c in point)
        yield point, tag
