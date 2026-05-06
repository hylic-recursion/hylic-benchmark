"""Render benchmark data into HTML fragments and text tables."""

from datetime import datetime


def heatmap_fragment(workloads, modes, data, title="", timestamp=None) -> str:
    """Produce a self-contained HTML <div> with a colored heatmap table.

    Design:
    - Green-to-red gradient by row (best → worst)
    - Best cell: gold border + star marker
    - All cells show absolute time + delta from best
    """
    lines = []
    if title:
        lines.append(f'<div class="bench-heatmap" data-title="{title}">')
    else:
        lines.append('<div class="bench-heatmap">')

    lines.append('<table>')
    lines.append('<tr><th class="wl-header">workload</th>')
    for m in modes:
        lines.append(f'<th>{m}</th>')
    lines.append('</tr>')

    for wl in workloads:
        row_vals = {m: data[(wl, m)] for m in modes if (wl, m) in data}
        if not row_vals:
            continue
        best = min(row_vals.values())
        worst = max(row_vals.values())

        lines.append(f'<tr><td class="wl">{wl}</td>')
        for m in modes:
            if (wl, m) not in data:
                lines.append('<td class="na">—</td>')
                continue

            v = data[(wl, m)]
            ratio = (v - best) / (worst - best) if worst > best else 0
            hue = 120 * (1 - ratio)
            sat = 55 + 15 * ratio  # more saturated as worse
            bg = f"hsl({hue:.0f}, {sat:.0f}%, 38%)"

            delta = ((v / best) - 1) * 100
            is_best = delta < 0.5

            if is_best:
                cls = "cell best"
                label = f"★ {v:.1f}ms"
            else:
                cls = "cell"
                label = f"{v:.1f}ms <span class='delta'>+{delta:.0f}%</span>"

            lines.append(f'<td class="{cls}" style="background:{bg}">{label}</td>')

        lines.append('</tr>')

    lines.append('</table>')
    if timestamp:
        lines.append(f'<p class="bench-timestamp">Generated: {timestamp}</p>')
    lines.append('</div>')
    return '\n'.join(lines)


def text_table(workloads, modes, data) -> str:
    """Plain text comparison table."""
    col_w = 18
    header = f"{'workload':<25}" + "".join(f"{m:>{col_w}}" for m in modes)
    lines = [header, "-" * len(header)]
    for wl in workloads:
        row_vals = {m: data[(wl, m)] for m in modes if (wl, m) in data}
        if not row_vals:
            continue
        best = min(row_vals.values())
        cells = []
        for m in modes:
            if (wl, m) in data:
                v = data[(wl, m)]
                delta = ((v / best) - 1) * 100
                if delta < 0.5:
                    cells.append(f"{v:>8.1f}ms  (best)")
                else:
                    cells.append(f"{v:>8.1f}ms (+{delta:.0f}%)")
            else:
                cells.append(f"{'n/a':>{col_w}}")
        lines.append(f"{wl:<25}" + "".join(f"{c:>{col_w}}" for c in cells))
    return "\n".join(lines) + "\n"


def csv_output(results) -> str:
    """CSV format."""
    lines = ["workload,mode,mean_ms"]
    for r in results:
        lines.append(f"{r['workload']},{r['mode']},{r['mean_ms']:.2f}")
    return "\n".join(lines) + "\n"


# CSS shared by all heatmap fragments — injected once by the loader
HEATMAP_CSS = """
.bench-heatmap table {
    border-collapse: collapse;
    font-family: 'SF Mono', 'Consolas', 'Monaco', monospace;
    font-size: 12px;
    margin: 8px 0 16px 0;
}
.bench-heatmap th {
    padding: 5px 10px;
    background: #2a2a2a;
    color: #aaa;
    border: 1px solid #444;
    font-weight: normal;
    white-space: nowrap;
}
.bench-heatmap th.wl-header {
    text-align: left;
}
.bench-heatmap td {
    padding: 4px 10px;
    border: 1px solid #444;
    text-align: right;
    color: #f0f0f0;
    white-space: nowrap;
}
.bench-heatmap td.wl {
    text-align: left;
    background: #1e1e1e;
    color: #999;
    font-weight: 600;
}
.bench-heatmap td.na {
    background: #1e1e1e;
    color: #555;
}
.bench-heatmap td.best {
    border: 2px solid #ffd700;
    font-weight: 700;
    color: #fff;
}
.bench-heatmap .delta {
    color: #ccc;
    font-size: 10px;
}
.bench-timestamp {
    font-size: 11px;
    color: #777;
    margin: 4px 0 0 0;
}
"""
