#!/bin/bash
set -o errexit
set -o pipefail
set -o nounset
# A helper script to generate _and visualize_ random L-Systems

./tools/random-production-rules.py $@ |
    tee /dev/tty |
    ./tools/parse.py --config - --iterations 6 |
    ./tools/interpret.py --log-level ERROR |
    ./tools/render.py
