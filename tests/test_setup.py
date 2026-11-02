from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def test_seed_files_present():
    readme = (ROOT / "README.md").read_text(encoding="utf-8")
    licence = (ROOT / "LICENSE").read_text(encoding="utf-8")
    gitignore = (ROOT / ".gitignore").read_text(encoding="utf-8")
    assert "# hookdoctor" in readme
    assert "A doctor for hooks" in readme
    assert "## Licence" in readme
    assert "MIT License" in licence
    assert "__pycache__/" in gitignore
    assert ".env" in gitignore
    assert "Closed rounds stand at 1 of 100" in readme


def test_agents_template_and_annex():
    agents = (ROOT / "AGENTS.md").read_text(encoding="utf-8")
    marker = "<!-- repo annex below this line -->"
    assert agents.startswith("# AGENTS.md")
    assert "This repo is one of the repos in David Engelmann's 39 repo storefront perfection program" in agents
    assert agents.count(marker) == 1
    annex = agents.split(marker, 1)[1]
    assert "Hookdoctor audits the harness's hooks, blindgate audits CI's checks" in annex
    assert "ingested as local files with synthetic fixtures only" in annex
    assert "no network in the tool or the tests" in annex
    assert "no real session data may ever become a fixture" in annex
    assert "The local gate is python -m pytest once the suite exists" in annex
    assert "closed round count stands in the README and moves only at a closure" in annex
