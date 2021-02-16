# geom2graph

A CLI application to convert geometries into a graph data structure.

---

Multi-geometries, with the exception of multi-geometries contained in a GEOMETRYCOLLECTION will be flattened,
with each component geometry being treated as distinct for the purposes of graph generation.

**TODO:** Figure out how to recursively flatten GEOMETRYCOLLECTIONs. Or add a simple Python script to flatten the WKT input.
