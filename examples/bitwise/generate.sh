#!/bin/bash
set -o errexit
set -o pipefail
set -o nounset
set -o noclobber

# BEGIN BITWISE_EXPR1
bitwise "(x & y) & (x ^ y) % 13" |
    wkt2svg --scale 10 --output ./examples/bitwise/expr1.svg
# END BITWISE_EXPR1
extract_snippet BITWISE_EXPR1

# BEGIN BITWISE_EXPR2
bitwise "(x & y) & (x ^ y) % 11" |
    wkt2svg --scale 10 --output ./examples/bitwise/expr2.svg
# END BITWISE_EXPR2
extract_snippet BITWISE_EXPR2

# BEGIN BITWISE_EXPR3
bitwise "(x & y) & (x ^ y) % 11" --neighbor-search-order south-west,south-east,south,east |
    wkt2svg --scale 10 --output ./examples/bitwise/expr3.svg
# END BITWISE_EXPR3
extract_snippet BITWISE_EXPR3
