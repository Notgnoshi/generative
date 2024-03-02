#!/bin/bash
set -o errexit
set -o pipefail
set -o nounset
set -o noclobber

debug "Generating random L-Systems"
# BEGIN RANDOM_LSYSTEMS_SNIPPET
for i in $(seq 0 13); do
    jq ".[$i]" examples/random-lsystems/saved.json |
        tools/parse-production-rules.py -c - -n "$(jq ".[$i].iterations" examples/random-lsystems/saved.json)" |
        tools/interpret-lstring.py -l ERROR -a "$(jq ".[$i].angle" examples/random-lsystems/saved.json)" |
        tools/project.py --scale "$(jq ".[$i].scale" examples/random-lsystems/saved.json)" --kind pca |
        wkt2svg --output "examples/random-lsystems/random-$i.svg"
done
# END RANDOM_LSYSTEMS_SNIPPET

extract_snippet RANDOM_LSYSTEMS_SNIPPET

debug "Generating markdown links"
RANDOM_LSYSTEM_IMAGES=""
for svg in examples/random-lsystems/random-*.svg; do
    LINK="![$svg]($svg)"
    RANDOM_LSYSTEM_IMAGES="$RANDOM_LSYSTEM_IMAGES\n\n$LINK"
done
