#!/bin/bash
set -o errexit
set -o pipefail
set -o nounset
set +o noclobber # Intentionally clobber stuff in /tmp

# BEGIN PACK_SNIPPET
cat examples/unit-square.wkt examples/unit-square.wkt examples/unit-square.wkt examples/unit-square.wkt |
    transform --scale=200 >/tmp/squares.wkt

pack --padding=10 --width=450 --height=450 </tmp/squares.wkt |
    wkt2svg --output=examples/pack/squares.svg
# END PACK_SNIPPET
extract_snippet PACK_SNIPPET
