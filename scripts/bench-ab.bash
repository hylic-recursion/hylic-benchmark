#!/usr/bin/env bash
#
# scripts/bench-ab.bash — run a benchmark across multiple git revisions of hylic.
#
# Usage: bench-ab.bash <bench-name> <output-base> [--archive <dir>] <label>=<git-ref>...
#
# For each label=ref pair:
#   1. Checks out hylic/src/ from the git ref (skipped for HEAD)
#   2. Runs bench-one.bash <bench-target> <run-dir>/<label>
#   3. Restores hylic/src/ to the starting state
#
# After all revisions: copies text tables to archive dir (if --archive given).
#
# Requires the hylic sibling repo to be reachable (workspace or ci mode);
# in single mode this script aborts.
#
# Env (passed through to bench-one.bash):
#   HYLIC_BENCH_SAMPLES, HYLIC_BENCH_WARMUP, HYLIC_BENCH_MEASURE

source "$(dirname "${BASH_SOURCE[0]}")/lib/paths.bash"
source "$(dirname "${BASH_SOURCE[0]}")/lib/common.bash"

require_repo hylic

bench_name="${1:?Usage: bench-ab.bash <bench-name> <output-base> [--archive <dir>] <label>=<ref>...}"
shift
output_base="${1:?Usage: bench-ab.bash <bench-name> <output-base> [--archive <dir>] <label>=<ref>...}"
shift

archive_dir=""
if [ "${1:-}" = "--archive" ]; then
    archive_dir="$2"
    shift 2
fi

revisions=("$@")
[ ${#revisions[@]} -gt 0 ] \
    || abort "no revisions supplied (need at least one <label>=<ref>)"

# Resolve output paths to absolute before any cd.
mkdir -p "$output_base"
output_base="$(cd "$output_base" && pwd)"
if [ -n "$archive_dir" ]; then
    mkdir -p "$archive_dir"
    archive_dir="$(cd "$archive_dir" && pwd)"
fi

timestamp="$(date +%Y%m%d-%H%M%S)"
run_dir="$output_base/$timestamp"
mkdir -p "$run_dir"

return_ref="$(git -C "$repo__hylic" rev-parse HEAD)"
return_branch="$(git -C "$repo__hylic" branch --show-current)"
current_label=""

require_clean_tree "$repo__hylic"

# ── Signal handling ─────────────────────────────────
interrupted() {
    sub_banner "INTERRUPTED at $(date +%H:%M:%S)"
    echo "  Processing: $current_label" >&2
    echo "  Restoring $repo__hylic/src/ to $return_branch ($return_ref)" >&2
    git -C "$repo__hylic" checkout "$return_ref" -- src/ 2>/dev/null || true
    echo "  Partial results in: $run_dir" >&2
    exit 130
}
trap interrupted INT TERM HUP

# ── Banner ──────────────────────────────────────────
banner "bench-ab: $bench_name across ${#revisions[@]} revisions"
log "hylic HEAD: $return_branch ($(git -C "$repo__hylic" rev-parse --short HEAD))"
log "samples=${HYLIC_BENCH_SAMPLES:-default} warmup=${HYLIC_BENCH_WARMUP:-default}s measure=${HYLIC_BENCH_MEASURE:-default}s"
log "run dir: $run_dir"
for spec in "${revisions[@]}"; do
    label="${spec%%=*}"
    ref="${spec#*=}"
    if [ "$ref" = "HEAD" ]; then
        log "  $label = HEAD ($(git -C "$repo__hylic" rev-parse --short HEAD))"
    else
        log "  $label = $ref ($(git -C "$repo__hylic" rev-parse --short "$ref" 2>/dev/null || echo '???'))"
    fi
done

# ── Main loop ───────────────────────────────────────
for spec in "${revisions[@]}"; do
    label="${spec%%=*}"
    ref="${spec#*=}"
    current_label="$label"

    sub_banner "$label"
    if [ "$ref" = "HEAD" ]; then
        log "$label — using current HEAD"
    else
        log "$label — checking out hylic/src/ from $ref"
        git -C "$repo__hylic" checkout "$ref" -- src/
    fi

    log "$label — benchmarking"
    bash "$repo__hylic_benchmark/scripts/bench-one.bash" "$bench_name" "$run_dir/$label"

    if [ "$ref" != "HEAD" ]; then
        log "$label — restoring hylic/src/"
        git -C "$repo__hylic" checkout "$return_ref" -- src/
    fi
    log "$label — done"
done

# ── Archive ─────────────────────────────────────────
if [ -n "$archive_dir" ]; then
    dest="$archive_dir/$timestamp"
    mkdir -p "$dest"
    for spec in "${revisions[@]}"; do
        label="${spec%%=*}"
        for f in "$run_dir/$label/report/"*.txt; do
            [ -f "$f" ] || continue
            cp "$f" "$dest/${label}--$(basename "$f")"
        done
    done
    log "archived to: $dest"
fi

trap - INT TERM HUP

# ── Summary ─────────────────────────────────────────
banner "bench-ab complete: $bench_name"
log "results: $run_dir"
for spec in "${revisions[@]}"; do
    label="${spec%%=*}"
    for f in "$run_dir/$label/report/"*.txt; do
        [ -f "$f" ] || continue
        sub_banner "$label"
        cat "$f" >&2
    done
done
