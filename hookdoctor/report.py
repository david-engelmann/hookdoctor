"""Deterministic rendering of an examination report."""

from __future__ import annotations

import json


def render_json(report):
    return json.dumps(report, indent=2, sort_keys=True) + "\n"


def _effects_line(effects):
    return (
        f"denial {effects['denial']}, modification {effects['modification']}, "
        f"consumed output {effects['consumed_output']}"
    )


def render_markdown(report):
    lines = []
    lines.append("# hookdoctor examination")
    lines.append("")
    lines.append(f"Format {report['format']}")
    lines.append(f"Configuration source {report['config_source']}")
    sources = ", ".join(report["session_sources"]) or "none"
    lines.append(f"Session sources {sources}")
    window = report["window"]
    lines.append(f"Window start {window['start']} end {window['end']}")
    lines.append("")
    lines.append("## Hooks")
    lines.append("")
    if not report["hooks"]:
        lines.append("No hooks are configured.")
        lines.append("")
    for entry in report["hooks"]:
        evidence = entry["evidence"]
        lines.append(f"### {entry['command']}")
        lines.append("")
        lines.append(f"Identifier {entry['id']}")
        lines.append(f"Event {entry['event']}")
        lines.append(f"Matcher {entry['matcher']}")
        lines.append(f"Class {entry['class']}")
        lines.append(f"Matched {evidence['matched']}")
        lines.append(f"Ran {evidence['ran']}")
        lines.append(f"Effects {_effects_line(evidence['effects'])}")
        lines.append(f"First ran at {evidence['first_ran_at']}")
        lines.append(f"Last ran at {evidence['last_ran_at']}")
        lines.append("")
    lines.append("## Near misses")
    lines.append("")
    if not report["near_misses"]:
        lines.append("None.")
        lines.append("")
    for miss in report["near_misses"]:
        lines.append(
            f"- Hook {miss['hook_id']} missed a {miss['event']} subject "
            f"at {miss['timestamp']} in {miss['source']} "
            f"on line {miss['line']}. {miss['detail']}."
        )
    if report["near_misses"]:
        lines.append("")
    return "\n".join(lines).rstrip("\n") + "\n"
