#!/bin/bash
set -o errexit
set -o pipefail
set -o nounset
set -o noclobber

BUNDLE_OUTPUT=$(
    # BEGIN BUNDLE_SNIPPET
    bundle <<EOF
    POINT(0 0)
    POINT(1 1)
EOF
    # END BUNDLE_SNIPPET
)
extract_snippet BUNDLE_SNIPPET 4
