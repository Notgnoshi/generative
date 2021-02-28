# geom2graph

A C++ CLI application to convert a set of WKT geometries to a graph representation.

## How to build

Ensure the submodule dependencies have been checked out, and use CMake.

```bash
git submodule update --init --recursive
mkdir build && cd build
cmake ..
make -j$(nproc)
```

## How to use

```bash
$ build/src/geom2graph --help
A C++ CLI application to convert a set of WKT geometries to a graph representation.
Usage:
  geom2graph [OPTION...]

  -h, --help           show this help message and exit
  -l, --log-level arg  TRACE, DEBUG, INFO, WARN, ERROR, or FATAL. (default: WARN)
  -i, --input arg      File to read WKT geometries from (default: -)
  -o, --output arg     File to write graph to (default: -)
  -t, --tolerance arg  Vertex snapping tolerance (default: 0.001)
```

```bash
$ cat examples/test-01.wkt
POINT(0 0)
LINESTRING(0 0, 1 1, 2 2)
LINESTRING(1 1.005, 0 1, 2 2.01)
$ build/src/geom2graph --input examples/test-01.wkt
0	POINT (0 0)
1	POINT (1 1)
2	POINT (2 2)
3	POINT (1 1.005)
4	POINT (0 1)
5	POINT (2 2.01)
#
0	1
1	2
3	4
4	5
```

```bash
$ cat examples/test-04.wkt
POLYGON((1 7, 5 7, 5 3, 1 3, 1 7))
POLYGON((3 5, 7 5, 7 1, 3 1, 3 5))
$ build/src/geom2graph --input examples/test-04.wkt
0	POINT (1 7)
1	POINT (5 7)
2	POINT (5 5)
3	POINT (5 3)
4	POINT (3 3)
5	POINT (1 3)
6	POINT (3 5)
7	POINT (7 5)
8	POINT (7 1)
9	POINT (3 1)
#
0	1
0	5
1	2
2	3
2	6
2	7
3	4
4	5
4	6
4	9
7	8
8	9
```

## Fuzzy vertex snapping

Vertices are snapped together within the specified tolerance.

```bash
$ build/src/geom2graph --input examples/test-04.wkt build/src/geom2graph --input examples/test-01.wkt --tolerance 0.2
0	POINT (0 0)
1	POINT (1 1)
2	POINT (2 2)
3	POINT (0 1)
#
0	1
1	2
1	3
2	3
```

## 3D output

Both 3D and 2D input is supported. When snapping vertices together, we assumed 2D coordinates have `z=0`.
The finer details of the output formatting still needs to be worked out; for now the output dimensionality will be mixed.

Notice in the following example that nodes #1 and #2 are adjacent, even with the dimensionality mismatch.

```bash
$ cat examples/test-03.wkt
LINESTRING Z(0 0 0, 1 1 1)
LINESTRING(0 0, 2 2)
LINESTRING(40 40, 50 50)
$ build/src/geom2graph --input examples/test-03.wkt --tolerance 0.2
0	POINT Z (0 0 0)
1	POINT Z (1 1 1)
2	POINT (2 2)
3	POINT (40 40)
4	POINT (50 50)
#
0	1
1	2
3	4
```
