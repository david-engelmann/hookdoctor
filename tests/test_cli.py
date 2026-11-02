import json
from pathlib import Path

from hookdoctor.cli import main

from synthetic_data import CONFIG, prompt_line, session_text, stop_line, tool_result_line, tool_use_line


def _write_inputs(tmp_path):
    config = tmp_path / "synthetic-settings.json"
    config.write_text(json.dumps(CONFIG), encoding="utf-8")
    session = tmp_path / "synthetic-session.jsonl"
    session.write_text(
        session_text(
            prompt_line(ts="2026-01-01T09:00:00Z"),
            tool_use_line("Bash", "call-1", "2026-01-01T09:00:05Z"),
            tool_result_line(
                "call-1", "2026-01-01T09:00:07Z", text="denied by policy", is_error=True
            ),
            stop_line("2026-01-01T09:01:00Z"),
        ),
        encoding="utf-8",
    )
    return config, session


def _run(tmp_path, name, config, session):
    output = tmp_path / f"{name}.json"
    report = tmp_path / f"{name}.md"
    code = main(
        [
            "--config",
            str(config),
            "--session",
            str(session),
            "--output",
            str(output),
            "--report",
            str(report),
        ]
    )
    return code, output, report


def test_cli_writes_deterministic_outputs(tmp_path):
    config, session = _write_inputs(tmp_path)
    code, output, report = _run(tmp_path, "first", config, session)
    assert code == 0
    first_json = output.read_bytes()
    first_md = report.read_bytes()
    code, output2, report2 = _run(tmp_path, "second", config, session)
    assert code == 0
    assert output2.read_bytes() == first_json
    assert report2.read_bytes() == first_md
    document = json.loads(first_json)
    assert document["format"] == "hookdoctor-report/1"
    assert document["config_source"] == "synthetic-settings.json"
    assert document["session_sources"] == ["synthetic-session.jsonl"]
    markdown = first_md.decode("utf-8")
    assert "guard-bash.sh" in markdown
    assert "firing" in markdown
    assert first_json.endswith(b"\n")


def test_missing_inputs_are_refused(tmp_path):
    config, session = _write_inputs(tmp_path)
    missing = tmp_path / "nope.json"
    code = main(
        [
            "--config",
            str(missing),
            "--session",
            str(session),
            "--output",
            str(tmp_path / "o.json"),
            "--report",
            str(tmp_path / "o.md"),
        ]
    )
    assert code == 2
    code = main(
        [
            "--config",
            str(config),
            "--session",
            str(missing),
            "--output",
            str(tmp_path / "o.json"),
            "--report",
            str(tmp_path / "o.md"),
        ]
    )
    assert code == 2


def test_outputs_carry_no_local_paths(tmp_path):
    config, session = _write_inputs(tmp_path)
    code, output, report = _run(tmp_path, "plain", config, session)
    assert code == 0
    for path in (output, report):
        text = path.read_text(encoding="utf-8")
        assert str(tmp_path) not in text
        assert str(Path.home()) not in text


def test_example_outputs_regenerate_byte_identical(tmp_path):
    root = Path(__file__).resolve().parents[1]
    output = tmp_path / "regen.json"
    report = tmp_path / "regen.md"
    code = main(
        [
            "--config",
            str(root / "examples" / "synthetic-settings.json"),
            "--session",
            str(root / "examples" / "synthetic-session.jsonl"),
            "--output",
            str(output),
            "--report",
            str(report),
        ]
    )
    assert code == 0
    assert output.read_bytes() == (root / "examples" / "hookdoctor-report.json").read_bytes()
    assert report.read_bytes() == (root / "examples" / "hookdoctor-report.md").read_bytes()
