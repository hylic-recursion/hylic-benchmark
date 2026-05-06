"""Collect criterion benchmark results into structured data."""

import json
from pathlib import Path

def collect_group(criterion_dir: Path, group_name: str) -> list[dict]:
    """Read criterion JSON estimates for a benchmark group."""
    results = []
    group_dir = criterion_dir / group_name
    if not group_dir.exists():
        return results

    for mode_dir in sorted(group_dir.iterdir()):
        if not mode_dir.is_dir():
            continue
        for workload_dir in sorted(mode_dir.iterdir()):
            if not workload_dir.is_dir():
                continue
            estimates = workload_dir / "new" / "estimates.json"
            if not estimates.exists():
                continue
            with open(estimates) as f:
                data = json.load(f)
            mean_ns = data["mean"]["point_estimate"]
            results.append({
                "mode": mode_dir.name,
                "workload": workload_dir.name,
                "mean_ms": mean_ns / 1_000_000,
            })
    return results


def group_timestamp(criterion_dir: Path, group_name: str) -> str:
    """Return the most recent mtime of any estimates.json in the group."""
    from datetime import datetime
    group_dir = criterion_dir / group_name
    if not group_dir.exists():
        return "unknown"
    max_mtime = 0.0
    for est in group_dir.rglob("new/estimates.json"):
        max_mtime = max(max_mtime, est.stat().st_mtime)
    if max_mtime == 0.0:
        return "unknown"
    return datetime.fromtimestamp(max_mtime).strftime("%Y-%m-%d %H:%M")


def pivot(results: list[dict], column_order: list[str] | None = None) -> tuple[list[str], list[str], dict]:
    """Pivot results into (workloads, modes, {(wl, mode): ms}).

    If column_order is provided, modes are returned in that order
    (missing modes omitted, extra modes appended at end).
    """
    workloads = []
    seen_modes = set()
    data = {}
    for r in results:
        wl, m = r["workload"], r["mode"]
        if wl not in workloads: workloads.append(wl)
        seen_modes.add(m)
        data[(wl, m)] = r["mean_ms"]

    if column_order:
        modes = [m for m in column_order if m in seen_modes]
        extra = sorted(m for m in seen_modes if m not in column_order)
        if extra:
            import sys
            print(f"  warning: unexpected modes not in column_order: {extra}", file=sys.stderr)
            modes.extend(extra)
    else:
        modes = sorted(seen_modes)

    return sorted(workloads), modes, data
