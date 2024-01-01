#!/bin/bash
set -o errexit
set -o pipefail
set -o nounset
set -o noclobber

cargo run --bin point-cloud -- \
    --points 80 \
    --domain=unit-square |
    cargo run --bin transform -- \
        --offset-x=-0.5 \
        --offset-y=-0.5 |
    cargo run --bin streamline -- \
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
    cargo run --bin wkt2svg -- \
        --scale 500 |
    display -

cargo run --bin point-cloud -- \
    --points 30 \
    --scale 2 \
    --domain=unit-square |
    cargo run --bin streamline -- \
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
    cargo run --bin wkt2svg -- \
        --scale 500 |
    display -
