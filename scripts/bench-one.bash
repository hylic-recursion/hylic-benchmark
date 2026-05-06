#!/usr/bin/env bash
#
# scripts/bench-one.bash — run one benchmark and render its report.
#
# Usage: bench-one.bash <bench-name> <output-dir>
#
# Produces:
#   <output-dir>/raw.txt       — full criterion output (also streamed to terminal)
#   <output-dir>/criterion/    — criterion JSON snapshot (no stale entries)
#   <output-dir>/report/       — text table, HTML heatmap, CSV
#
# Env (defaults shown):
#   HYLIC_BENCH_SAMPLES=80
#   HYLIC_BENCH_WARMUP=4
#   HYLIC_BENCH_MEASURE=40

source "$(dirname "${BASH_SOURCE[0]}")/lib/paths.bash"
source "$(dirname "${BASH_SOURCE[0]}")/lib/common.bash"

bench_name="${1:?Usage: bench-one.bash <bench-name> <output-dir>}"
output_dir="${2:?Usage: bench-one.bash <bench-name> <output-dir>}"

mkdir -p "$output_dir"
output_dir="$(cd "$output_dir" && pwd)"

cd "$repo__hylic_benchmark"

sample_size="${HYLIC_BENCH_SAMPLES:-80}"
warm_up="${HYLIC_BENCH_WARMUP:-4}"
measure="${HYLIC_BENCH_MEASURE:-40}"

# Group name as criterion sees it: bench_<x>_<y> → x-y
group_name="$(echo "$bench_name" | sed 's/^bench_//' | tr '_' '-')"

# Wherever cargo decides target_directory is — works in workspace or standalone.
target_dir="$(cargo metadata --no-deps --format-version 1 \
              | python3 -c 'import json, sys; print(json.load(sys.stdin)["target_directory"])')"
criterion_dir="$target_dir/criterion"

banner "$bench_name → $output_dir"
log "samples=$sample_size warmup=${warm_up}s measure=${measure}s mode=$MODE"

# Drop stale criterion entries for this group (criterion accumulates;
# renamed runners would otherwise persist as ghosts).
rm -rf "$criterion_dir/$group_name"

# Run benchmark — stream to terminal AND save to file.
cargo bench -p hylic-benchmark --bench "$bench_name" -- \
    --sample-size "$sample_size" \
    --warm-up-time "$warm_up" \
    --measurement-time "$measure" \
    2>&1 | tee "$output_dir/raw.txt"

# Snapshot criterion JSON next to the raw output.
[ -d "$criterion_dir/$group_name" ] \
    || abort "criterion did not produce $criterion_dir/$group_name"
rm -rf "$output_dir/criterion"
mkdir -p "$output_dir/criterion"
cp -r "$criterion_dir/$group_name" "$output_dir/criterion/$group_name"

# Render the report from the snapshot.
mkdir -p "$output_dir/report"
python3 "$repo__hylic_benchmark/scripts/bench-report.py" \
    --group "$group_name" \
    --criterion-dir "$output_dir/criterion" \
    --output-dir "$output_dir/report" \
    2>&1 | tee -a "$output_dir/raw.txt"

log "$bench_name complete"
