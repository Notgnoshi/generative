#!/bin/bash
set -o errexit
set -o pipefail
set -o nounset
set -o noclobber

debug "Generating clifford ..."
attractor \
    --seed 1193803725491079949 \
    --script ./examples/attractor/clifford.rn \
    --output ./examples/attractor/clifford.png \
    --output-format image \
    --num-points 400 \
    --iterations 800

debug "Generating ikeda ..."
attractor \
    --seed 14245741203239691500 \
    --script ./examples/attractor/ikeda.rn \
    --output ./examples/attractor/ikeda.png \
    --output-format image \
    --num-points 500 \
    --iterations 500

debug "Generating johnny-svensson ..."
attractor \
    --seed 2310402659768744900 \
    --script ./examples/attractor/johnny-svensson.rn \
    --output ./examples/attractor/johnny-svensson.png \
    --output-format image \
    --num-points 200 \
    --iterations 800

debug "Generating peter-de-jong ..."
attractor \
    --seed 10329922707181609977 \
    --script ./examples/attractor/peter-de-jong.rn \
    --output ./examples/attractor/peter-de-jong.png \
    --output-format image \
    --num-points 200 \
    --iterations 800

debug "Generating tinkerbell ..."
# BEGIN TINKERBELL_SNIPPET
attractor \
    --script ./examples/attractor/tinkerbell.rn \
    --output ./examples/attractor/tinkerbell.png \
    --output-format image \
    -x=-0.72 \
    -y=-0.64 \
    --iterations 500000
# END TINKERBELL_SNIPPET
extract_snippet TINKERBELL_SNIPPET

debug "Generating fractal-dreams-ssss ..."
attractor \
    --seed 4392994853744049110 \
    --script ./examples/attractor/fractal-dreams-ssss.rn \
    --output ./examples/attractor/fractal-dreams-ssss.png \
    --output-format image \
    --num-points 1000 \
    --iterations 2000

debug "Generating gumowski-mira ..."
attractor \
    --seed 6844197751594810350 \
    --script ./examples/attractor/gumowski-mira.rn \
    --output ./examples/attractor/gumowski-mira.png \
    --output-format image \
    --num-points 1000 \
    --iterations 5000
