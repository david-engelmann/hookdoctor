"""Command line entry point for hookdoctor."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

from .errors import HookdoctorError
from .examine import build_report, validate_report
from .ingest import parse_config, parse_session_text
from .report import render_json, render_markdown


def _read_bytes(path, label):
    try:
        return Path(path).read_bytes()
    except OSError as exc:
        raise HookdoctorError(f"the {label} could not be read") from exc


def _read_json(path, label):
    data = _read_bytes(path, label)
    try:
        document = json.loads(data.decode("utf-8"))
    except (UnicodeDecodeError, json.JSONDecodeError) as exc:
        raise HookdoctorError(f"the {label} is not valid JSON") from exc
    return document


def _read_text(path, label):
    data = _read_bytes(path, label)
    try:
        return data.decode("utf-8")
    except UnicodeDecodeError as exc:
        raise HookdoctorError(f"the {label} is not UTF-8 text") from exc


def _write_text(path, text):
    destination = Path(path)
    try:
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_text(text, encoding="utf-8")
    except OSError as exc:
        raise HookdoctorError("an output could not be written") from exc


def build_from_args(args):
    document = _read_json(args.config, "configuration")
    hooks = parse_config(document)
    records = []
    for index, session_path in enumerate(args.session):
        text = _read_text(session_path, "session file")
        records.extend(parse_session_text(text, file_index=index))
    if args.session_label is not None and len(args.session_label) != len(args.session):
        raise HookdoctorError("session labels do not match the session files")
    config_source = args.config_label or Path(args.config).name
    session_sources = args.session_label or [
        Path(path).name for path in args.session
    ]
    report = build_report(
        hooks,
        records,
        config_source,
        session_sources,
        window_start=args.window_start,
        window_end=args.window_end,
    )
    validate_report(
        report,
        hooks,
        records,
        config_source,
        session_sources,
        window_start=args.window_start,
        window_end=args.window_end,
    )
    return report


def run(args):
    report = build_from_args(args)
    _write_text(args.output, render_json(report))
    _write_text(args.report, render_markdown(report))
    return 0


def main(argv=None):
    parser = argparse.ArgumentParser(
        prog="hookdoctor",
        description="Examine configured hooks against session execution traces.",
    )
    parser.add_argument("--config", required=True, help="local hooks settings JSON")
    parser.add_argument(
        "--session",
        action="append",
        required=True,
        help="local session JSONL file, repeatable",
    )
    parser.add_argument("--output", required=True, help="JSON report output path")
    parser.add_argument("--report", required=True, help="markdown report output path")
    parser.add_argument("--config-label", help="configuration source label")
    parser.add_argument(
        "--session-label",
        action="append",
        help="session source label, repeatable, in session order",
    )
    parser.add_argument("--window-start", help="inclusive window start timestamp")
    parser.add_argument("--window-end", help="exclusive window end timestamp")
    args = parser.parse_args(argv)
    try:
        return run(args)
    except HookdoctorError as exc:
        print(f"hookdoctor error: {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
