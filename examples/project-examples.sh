#!/bin/bash
set -x

../tools/parse.py --config fractal-plant-3d.json | ../tools/interpret.py --stepsize=3 --angle=22.5 >/tmp/plant.wkt
for projection in tsne isomap lle; do
    ../tools/project.py --kind=$projection --input /tmp/plant.wkt | ../tools/wkt2svg.py -o plant-$projection.svg
done
