#!/bin/bash
set -o errexit
set -o pipefail
set -o nounset
set -o noclobber

PARSE_LSYSTEM_OUTPUT=$(
    # BEGIN PARSE_LSYSTEM_SNIPPET
    ./tools/parse-production-rules.py --rule 'a -> ab' --rule 'b -> a' --axiom a --iterations 3
    # END PARSE_LSYSTEM_SNIPPET
)
extract_snippet PARSE_LSYSTEM_SNIPPET 4

RANDOM_LSYSTEM_RULES_OUTPUT=$(
    # BEGIN RANDOM_LSYSTEM_RULES_SNIPPET
    ./tools/random-production-rules.py --seed 4290989563 |
        ./tools/parse-production-rules.py --config - --iterations 3
    # END RANDOM_LSYSTEM_RULES_SNIPPET
)
extract_snippet RANDOM_LSYSTEM_RULES_SNIPPET 4

SIERPINKSI_TREE_OUTPUT=$(
    # BEGIN SIERPINKSI_TREE_SNIPPET
    ./tools/parse-production-rules.py --config ./examples/lsystems/sierpinski-tree.json |
        ./tools/interpret-lstring.py |
        tail -n 4
    # END SIERPINKSI_TREE_SNIPPET
)
extract_snippet SIERPINKSI_TREE_SNIPPET 4

# BEGIN SIERPINSKI_TREE_SVG
./tools/parse-production-rules.py --config ./examples/lsystems/sierpinski-tree.json |
    ./tools/interpret-lstring.py |
    ./tools/project.py --kind=yz |
    wkt2svg --output ./examples/lsystems/sierpinski-tree.svg
# END SIERPINSKI_TREE_SVG
extract_snippet SIERPINSKI_TREE_SVG
