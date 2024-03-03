#!/bin/bash
set -o errexit
set -o pipefail
set -o nounset
set -o noclobber

# BEGIN STREAMLINE_SNIPPET1
point-cloud \
    --points 80 \
    --seed=4628778017671551752 \
    --domain=unit-square |
    transform \
        --offset-x=-0.5 \
        --offset-y=-0.5 |
    streamline \
        --min-x=-0.6 \
        --max-x=0.7 \
        --min-y=-1 \
        --max-y=1 \
        --delta-h=0.1 \
        --time-steps=20 \
        --function "let temp = sqrt(x ** 2.0 + y ** 2.0 + 4.0); x = -sin(x) / temp; y = y / temp;" \
        --draw-vector-field \
        --vector-field-style="STROKE(gray)" \
        --vector-field-style="STROKEDASHARRAY(1)" \
        --streamline-style="STROKE(black)" \
        --streamline-style="STROKEDASHARRAY(0)" \
        --draw-geometries \
        --geometry-style="STROKE(red)" |
    wkt2svg \
        --scale 500 --output ./examples/streamline/field1.svg
# END STREAMLINE_SNIPPET1
extract_snippet STREAMLINE_SNIPPET1

# BEGIN STREAMLINE_SNIPPET2
point-cloud \
    --seed=5882435996591106192 \
    --points 30 \
    --scale 2 \
    --domain=unit-square |
    streamline \
        --seed=192545950949821414 \
        --max-x=2.0 \
        --max-y=2.0 \
        --delta-h=0.1 \
        --delta-t=0.05 \
        --time-steps=20 \
        --draw-vector-field \
        --vector-field-style="STROKE(gray)" \
        --vector-field-style="STROKEDASHARRAY(1)" \
        --streamline-style="STROKE(black)" \
        --streamline-style="STROKEDASHARRAY(0)" \
        --draw-geometries \
        --geometry-style="STROKE(red)" |
    wkt2svg \
        --scale 500 --output ./examples/streamline/field2.svg
# END STREAMLINE_SNIPPET2
extract_snippet STREAMLINE_SNIPPET2
