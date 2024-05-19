#!/bin/bash
# shellcheck disable=SC1090,SC1091
set -o errexit
set -o pipefail
set -o nounset
set -o noclobber

RED=$(tput setaf 1)
GREEN=$(tput setaf 2)
YELLOW=$(tput setaf 3)
BLUE=$(tput setaf 4)
RESET=$(tput sgr0)

debug() {
    echo "${BLUE}DEBUG:${RESET} $*"
}

info() {
    echo "${GREEN}INFO:${RESET} $*"
}

warn() {
    echo "${YELLOW}WARN:${RESET} $*"
}

error() {
    echo "${RED}ERROR:${RESET} $*" >&2
}

usage() {
    echo "Usage: $0 [--help] [[EXAMPLE_STEP] ...]"
    echo
    echo "Generate the example data, and the project README.md"
    echo
    echo "  --help, -h      Show this help and exit"
    echo
    echo "  EXAMPLE_STEP    One of the examples/*/*.sh scripts"
}

setup_environment() {
    info "Setting up environment ..."
    local this_script_source="${BASH_SOURCE[0]}"
    local repo_root
    repo_root="$(cd -P "$(dirname "$this_script_source")" >/dev/null 2>&1 && pwd)/"
    repo_root="$(readlink --canonicalize --no-newline "${repo_root}")"
    cd "$repo_root"
    if [[ ! -f .venv/bin/activate ]]; then
        info "Python virtualenv not found. Attempting to create"
        python -m venv --prompt generative .venv
        source .venv/bin/activate
        python -m pip install -r requirements.txt
    else
        source .venv/bin/activate
    fi

    cargo build --release
    export PATH="$repo_root/target/release/:$PATH"
}

extract_snippet() {
    local snippet="$1"
    local gobble_leading_whitespace="${2:-0}"
    local source_file="${3:-${BASH_SOURCE[1]}}"

    # Extract the contents between '# BEGIN $snippet' and '# END $snippet'
    local snippet_contents
    snippet_contents="$(sed -En "/^[[:space:]]*# BEGIN $snippet\\b/,/^[[:space:]]*# END $snippet\\b/p" "$source_file" | sed '1d;$d')"
    snippet_contents="$(echo -e "$snippet_contents" | sed -E "s/^[[:space:]]{$gobble_leading_whitespace}//")"

    # Populate the variable with name '$snippet' with '$snippet_contents'
    printf -v "$snippet" '%s' "$snippet_contents"
}

configure() {
    local template="$1"
    local final_output="$2"
    local dry_run="${3:-false}"
    local temp_output
    temp_output="$(mktemp "$template.$final_output.XXXXXXXXX")"
    # shellcheck disable=SC2064
    trap "rm '$temp_output'" EXIT
    info "Generating '$final_output' from template '$template'"

    cp "$template" "$temp_output"

    local placeholders
    placeholders=$(grep --only-matching --extended-regexp '@[a-zA-Z_]+[a-zA-Z0-9_]*@' "$temp_output" || true)
    for placeholder in $placeholders; do
        local placeholder_var="${placeholder//@/}"
        if [[ -n ${!placeholder_var+x} ]]; then
            local placeholder_value="${!placeholder_var}"
            placeholder_value="${placeholder_value//\\/\\\\}"
            placeholder_value="${placeholder_value//$'\n'/'\n'}"
            # debug "Found placeholder '$placeholder' with value '$placeholder_value'"
            debug "Processing placeholder '$placeholder' ..."
            sed -i "s~$placeholder~${placeholder_value}~" "$temp_output"
        else
            warn "Could not find substitution for '$placeholder'"
        fi
    done

    if [[ "$dry_run" = "false" ]]; then
        cp "$temp_output" "$final_output"
    fi
}

main() {
    local examples=()

    while [[ $# -gt 0 ]]; do
        case "$1" in
        --help | -h)
            usage
            exit 0
            ;;
        -*)
            error "Unexpected option: $1"
            usage >&2
            exit 1
            ;;
        *)
            examples+=("$1")
            ;;
        esac
        shift
    done

    setup_environment

    # If example scripts have been passed manually, run just them, otherwise run everything.
    if [[ "${#examples[@]}" -gt 0 ]]; then
        for example in "${examples[@]}"; do
            if [[ ! -f "$example" ]]; then
                error "Example '$example' doesn't exist"
                exit 1
            fi

            info "Generating example: $example ..."
            source "$example"
        done
    else
        shopt -s globstar
        for example in ./examples/**/*.sh; do
            info "Generating example: $example ..."
            source "$example"
        done
    fi
    debug "Finished generating examples"

    # Generate the README at the end, so that we don't leave it in a broken state, and so that we
    # can optionally run just a single example.
    configure README.md.in README.md
}

main "$@"
