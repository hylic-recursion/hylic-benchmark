"""Funnel axes analysis: parse names, emit structured JSON for the interactive view."""

import json
from pathlib import Path

# The three funnel axes
QUEUE_AXIS = ("pw", "sh")
ACCUMULATE_AXIS = ("arrv", "fin")
WAKE_AXIS = ("push", "batch", "k4", "k2")

AXIS_NAMES = ["queue", "accumulate", "wake"]
AXIS_VALUES = [QUEUE_AXIS, ACCUMULATE_AXIS, WAKE_AXIS]

BASELINE = "real.rayon"


def parse_funnel_name(name: str) -> dict | None:
    """Parse 'funnel.pw.arrv.push' → {queue: 'pw', accumulate: 'arrv', wake: 'push'}.
    Returns None if not a funnel variant."""
    if not name.startswith("funnel."):
        return None
    parts = name.split(".")
    if len(parts) != 4:
        return None
    _, queue, acc, wake = parts
    if queue in QUEUE_AXIS and acc in ACCUMULATE_AXIS and wake in WAKE_AXIS:
        return {"queue": queue, "accumulate": acc, "wake": wake}
    return None


def build_analysis_json(workloads: list[str], modes: list[str],
                        data: dict[tuple[str, str], float]) -> dict:
    """Build the structured JSON for the analysis view."""

    baseline_values = {}
    for wl in workloads:
        baseline_values[wl] = data.get((wl, BASELINE))

    non_funnel = {}
    funnel = {}

    for mode in modes:
        axes = parse_funnel_name(mode)
        values = {}
        for wl in workloads:
            if (wl, mode) in data:
                values[wl] = round(data[(wl, mode)], 4)

        if axes:
            key = f"{axes['queue']}.{axes['accumulate']}.{axes['wake']}"
            funnel[key] = {"axes": axes, "values": values}
        else:
            non_funnel[mode] = values

    return {
        "baseline": BASELINE,
        "baseline_values": {wl: round(v, 4) for wl, v in baseline_values.items() if v},
        "workloads": workloads,
        "axis_names": AXIS_NAMES,
        "axis_values": {
            "queue": list(QUEUE_AXIS),
            "accumulate": list(ACCUMULATE_AXIS),
            "wake": list(WAKE_AXIS),
        },
        "non_funnel": non_funnel,
        "funnel": funnel,
    }


def emit_analysis(workloads, modes, data, output_dir: Path, timestamp: str):
    """Write matrix-analysis.json and matrix-analysis.html."""
    analysis = build_analysis_json(workloads, modes, data)
    analysis["timestamp"] = timestamp

    json_path = output_dir / "matrix-analysis.json"
    json_path.write_text(json.dumps(analysis, indent=2))

    # Copy the template HTML (it loads the JSON via fetch)
    template = Path(__file__).parent / "template.html"
    html_path = output_dir / "matrix-analysis.html"
    html_path.write_text(template.read_text())

    print(f"  analysis: {json_path.name} + {html_path.name}")
