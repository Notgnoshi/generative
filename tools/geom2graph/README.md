# geom2graph

A CLI application to convert geometries into a graph data structure.

---

For reasons I don't fully understand, the current implementation requires an unstable feature.
Install the nightly release and set it as the project default:

```
rustup toolchain install nightly
# In the generative/tools/geom2graph/ directory.
rustup override set nightly
```

**TODO:** I don't think GATs are _actually_ necessary. Fix.

---

Multi-geometries, with the exception of multi-geometries contained in a GEOMETRYCOLLECTION will be flattened,
with each component geometry being treated as distinct for the purposes of graph generation.

**TODO:** Figure out how to recursively flatten GEOMETRYCOLLECTIONs. Or add a simple Python script to flatten the WKT input.
