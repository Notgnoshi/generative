#!/bin/bash
set -o errexit
set -o pipefail
set -o nounset
set -o noclobber

POINT_CLOUD_OUTPUT=$(
    # BEGIN POINT_CLOUD_SNIPPET
    point-cloud --points 4 --domain unit-circle --scale 100 --seed 15838575381579332872
    # END POINT_CLOUD_SNIPPET
)
extract_snippet POINT_CLOUD_SNIPPET 4
