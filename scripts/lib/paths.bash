# scripts/lib/paths.bash — single source of truth for paths and run mode.
#
# Sourced by every script in scripts/ via:
#   source "$(dirname "${BASH_SOURCE[0]}")/lib/paths.bash"
#
# After sourcing, the following are set:
#
#   $repo__hylic_benchmark          — this repo (always reachable)
#   $MODE                           — workspace | ci | single
#   $repo__hylic                    — set when reachable (workspace, ci)
#   $repo__hylic_pipeline           — set when reachable (workspace, ci)
#   $repo__hylic_docs               — set when reachable (workspace, ci)
#   $docs__bench_results            — set when $repo__hylic_docs is set
#
# Mode-detection rule (one-way, no fallbacks):
#   - $GITHUB_WORKSPACE set                 → mode=ci,        repos under it
#   - parent-of-repo has hylic-docs sibling → mode=workspace, repos under parent
#   - otherwise                             → mode=single,    only this repo known
#
# `set -e` and other strict flags are installed in common.bash; this
# file installs no shell options.

_paths_lib_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
declare -gr repo__hylic_benchmark="$(cd "$_paths_lib_dir/../.." && pwd)"
unset _paths_lib_dir

if [ -n "${GITHUB_WORKSPACE:-}" ]; then
    declare -gr MODE=ci
    _repos_parent="$GITHUB_WORKSPACE"
elif [ -d "$repo__hylic_benchmark/../hylic-docs" ]; then
    declare -gr MODE=workspace
    _repos_parent="$(cd "$repo__hylic_benchmark/.." && pwd)"
else
    declare -gr MODE=single
    _repos_parent=""
fi

if [ -n "$_repos_parent" ]; then
    [ -d "$_repos_parent/hylic" ]          && declare -gr repo__hylic="$_repos_parent/hylic"
    [ -d "$_repos_parent/hylic-pipeline" ] && declare -gr repo__hylic_pipeline="$_repos_parent/hylic-pipeline"
    [ -d "$_repos_parent/hylic-docs" ]     && declare -gr repo__hylic_docs="$_repos_parent/hylic-docs"
fi
unset _repos_parent

if [ -n "${repo__hylic_docs:-}" ]; then
    declare -gr docs__bench_results="$repo__hylic_docs/book/src/bench-results"
fi

export MODE repo__hylic_benchmark
[ -n "${repo__hylic:-}" ]          && export repo__hylic
[ -n "${repo__hylic_pipeline:-}" ] && export repo__hylic_pipeline
[ -n "${repo__hylic_docs:-}" ]     && export repo__hylic_docs
[ -n "${docs__bench_results:-}" ]  && export docs__bench_results
