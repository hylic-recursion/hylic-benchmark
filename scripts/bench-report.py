#!/usr/bin/env python3
"""Generate benchmark report artifacts: per-group HTML fragments, CSV, text tables.

Each benchmark group produces:
  {group}.html   — self-contained HTML fragment (div with heatmap table)
  {group}.txt    — plain text table
  {group}.csv    — raw data

Plus a shared bench-style.css for the heatmap styling.

Usage:
  bench-report.py --criterion-dir <dir> --output-dir <dir>
  bench-report.py --criterion-dir <dir> --output-dir <dir> --group matrix

Publication to hylic-docs/book/src/bench-results/ is a separate step
(scripts/publish-to-docs.bash); this script does not write outside
--output-dir.
"""

import argparse, sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))

# Defaults assume hylic-benchmark standalone layout. In workspace mode,
# bench-one.bash passes --criterion-dir explicitly (cargo writes to the
# workspace target, not the per-crate target).
_REPO_ROOT = Path(__file__).parent.parent
DEFAULT_CRITERION_DIR = _REPO_ROOT / "target" / "criterion"
DEFAULT_OUTPUT_DIR = _REPO_ROOT / "target" / "bench-report"

# Column order matches the runner definition in runners.rs / baselines.rs.
# This is the authoritative display order for each benchmark suite.

MATRIX_COLUMNS = [
    # Handrolled baselines
    "hand.rayon", "hand.pool", "real.rayon",
    # Framework competitors
    "rayon", "sheque",
    # Funnel: PerWorker + OnArrival
    "funnel.pw.arrv.push", "funnel.pw.arrv.batch", "funnel.pw.arrv.k4", "funnel.pw.arrv.k2",
    # Funnel: Shared + OnArrival
    "funnel.sh.arrv.push", "funnel.sh.arrv.batch", "funnel.sh.arrv.k4", "funnel.sh.arrv.k2",
    # Funnel: PerWorker + OnFinalize
    "funnel.pw.fin.push", "funnel.pw.fin.batch", "funnel.pw.fin.k4", "funnel.pw.fin.k2",
    # Funnel: Shared + OnFinalize
    "funnel.sh.fin.push", "funnel.sh.fin.batch", "funnel.sh.fin.k4", "funnel.sh.fin.k2",
]

MODSIM_COLUMNS = [
    # Handrolled baseline
    "vanilla.rayon",
    # Framework competitors
    "rayon", "sheque",
    # Funnel: PerWorker + OnArrival
    "funnel.pw.arrv.push", "funnel.pw.arrv.batch", "funnel.pw.arrv.k4", "funnel.pw.arrv.k2",
    # Funnel: Shared + OnArrival
    "funnel.sh.arrv.push", "funnel.sh.arrv.batch", "funnel.sh.arrv.k4", "funnel.sh.arrv.k2",
    # Funnel: PerWorker + OnFinalize
    "funnel.pw.fin.push", "funnel.pw.fin.batch", "funnel.pw.fin.k4", "funnel.pw.fin.k2",
    # Funnel: Shared + OnFinalize
    "funnel.sh.fin.push", "funnel.sh.fin.batch", "funnel.sh.fin.k4", "funnel.sh.fin.k2",
]

OVERHEAD_COLUMNS = [
    "fused", "hand.seq", "real.seq",
]

QUICK_COLUMNS = [
    "real.rayon",
    "funnel.pw.arrv.k4", "funnel.sh.arrv.k4",
    "funnel.pw.fin.k4", "funnel.sh.fin.k4",
]

GROUPS = [
    ("quick",     "quick",     "Quick — WIP improvement tracker",     QUICK_COLUMNS),
    ("overhead",  "overhead",  "Overhead — framework cost",          OVERHEAD_COLUMNS),
    ("matrix",    "matrix",    "Matrix — full executor comparison",   MATRIX_COLUMNS),
    ("modsim",    "modsim",    "Module simulation — realistic workload", MODSIM_COLUMNS),
]

def main():
    from bench_report.collect import collect_group, group_timestamp, pivot
    from bench_report.render import heatmap_fragment, text_table, csv_output, HEATMAP_CSS

    parser = argparse.ArgumentParser()
    parser.add_argument('--group', help='Process one group only (by dir name)')
    parser.add_argument('--criterion-dir', type=Path, default=DEFAULT_CRITERION_DIR,
                        help='Criterion results directory')
    parser.add_argument('--output-dir', type=Path, default=DEFAULT_OUTPUT_DIR,
                        help='Output directory for report artifacts')
    args = parser.parse_args()

    criterion_dir = args.criterion_dir
    output_dir = args.output_dir

    groups = GROUPS
    if args.group:
        groups = [(fp, dn, t, co) for fp, dn, t, co in GROUPS if dn == args.group]
        if not groups:
            print(f"Unknown group: {args.group}", file=sys.stderr)
            print(f"Available: {', '.join(dn for _, dn, _, _ in GROUPS)}", file=sys.stderr)
            sys.exit(1)

    output_dir.mkdir(parents=True, exist_ok=True)
    (output_dir / "bench-style.css").write_text(HEATMAP_CSS)

    found_any = False
    for file_prefix, dir_name, title, column_order in groups:
        results = collect_group(criterion_dir, dir_name)
        if not results:
            print(f"  (no results for {dir_name})", file=sys.stderr)
            continue
        found_any = True

        workloads, modes, data = pivot(results, column_order)
        timestamp = group_timestamp(criterion_dir, dir_name)

        html = heatmap_fragment(workloads, modes, data, title=title, timestamp=timestamp)
        (output_dir / f"{file_prefix}.html").write_text(html)

        txt = text_table(workloads, modes, data)
        (output_dir / f"{file_prefix}.txt").write_text(txt)
        print(f"=== {title} ===")
        print(txt)

        # Emit interactive analysis for the matrix group
        if dir_name == "matrix":
            from bench_report.analysis.axes import emit_analysis
            emit_analysis(workloads, modes, data, output_dir, timestamp)

        (output_dir / f"{file_prefix}.csv").write_text(csv_output(results))

    if not found_any:
        print("No benchmark results found.", file=sys.stderr)
        sys.exit(1)

    print(f"Report artifacts in: {output_dir}")

if __name__ == "__main__":
    main()
