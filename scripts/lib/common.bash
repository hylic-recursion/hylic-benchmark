# scripts/lib/common.bash — diagnostics, control helpers, repo assertions.
#
# Sourced after paths.bash. Provides:
#   banner / sub_banner / log / abort
#   require_repo <name> — aborts if a sibling repo isn't reachable in this mode
#   require_clean_tree <repo-path> — aborts if hylic/src/ has uncommitted changes
#
# Style: stderr for diagnostics, exit 1 from abort. Strict bash mode is
# installed here so every script that sources this gets `set -euo pipefail`.

set -euo pipefail

banner() {
    echo "" >&2
    echo "══════════════════════════════════════════════════════════════" >&2
    echo " $*" >&2
    echo "══════════════════════════════════════════════════════════════" >&2
}

sub_banner() {
    echo "" >&2
    echo "──────────────────────────────────────────────────────────────" >&2
    echo " $*" >&2
    echo "──────────────────────────────────────────────────────────────" >&2
}

log() {
    echo "[$(date +%H:%M:%S)] $*" >&2
}

abort() {
    echo "" >&2
    echo "ABORT: $*" >&2
    exit 1
}

# Asserts that a sibling repo is reachable in this MODE. The argument is
# the repo's short name (e.g. "hylic-docs" → checks $repo__hylic_docs).
require_repo() {
    local name="$1"
    local var="repo__${name//-/_}"
    [ -n "${!var:-}" ] || abort "$name not reachable in mode=$MODE; this script needs the workspace or ci layout"
}

require_clean_tree() {
    local path="$1"
    git -C "$path" diff --quiet HEAD -- src/ \
        || abort "$path/src/ has uncommitted changes; commit or stash first"
}
