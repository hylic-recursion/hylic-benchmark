#!/usr/bin/env bash
#
# scripts/publish-to-docs.bash — copy bench reports to hylic-docs.
#
# Usage:
#   publish-to-docs.bash             # publish every bench under target/bench-latest/
#   publish-to-docs.bash <bench>     # publish only <bench> (e.g. matrix, modsim)
#
# Requires hylic-docs to be reachable (workspace or ci mode). In single
# mode (standalone hylic-benchmark clone) the script aborts.

source "$(dirname "${BASH_SOURCE[0]}")/lib/paths.bash"
source "$(dirname "${BASH_SOURCE[0]}")/lib/common.bash"

require_repo hylic-docs

bench_filter="${1:-}"

bench_root="$repo__hylic_benchmark/target/bench-latest"
[ -d "$bench_root" ] || abort "no bench output at $bench_root (run 'make bench-<name>' first)"

mkdir -p "$docs__bench_results"

published=0
for d in "$bench_root"/*/; do
    bench="$(basename "$d")"
    [ -d "$d/report" ] || continue
    if [ -n "$bench_filter" ] && [ "$bench" != "$bench_filter" ]; then
        continue
    fi
    log "publishing $bench → $docs__bench_results/"
    cp "$d/report/"* "$docs__bench_results/"
    published=$((published + 1))
done

[ "$published" -gt 0 ] \
    || abort "nothing matched filter '$bench_filter' under $bench_root"
log "published $published bench(es) to $docs__bench_results"
