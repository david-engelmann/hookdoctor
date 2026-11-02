import re
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def _suite_test_names():
    names = set()
    for path in (ROOT / "tests").glob("test_*.py"):
        names.update(re.findall(r"def (test_[a-z0-9_]+)\(", path.read_text(encoding="utf-8")))
    return names


def test_every_contract_proof_name_is_a_real_test():
    contract = (ROOT / "CONTRACT.md").read_text(encoding="utf-8")
    proven = set(re.findall(r"Proven by ([a-z0-9_ ,]+)\.", contract))
    named = set()
    for chunk in proven:
        for part in chunk.split(","):
            named.update(name.strip() for name in part.split(" and "))
    assert named, "the contract names no proving tests"
    missing = named - _suite_test_names()
    assert missing == set(), f"contract names tests that do not exist: {sorted(missing)}"


def test_public_files_carry_no_em_or_en_dashes():
    for name in ("CONTRACT.md", "README.md", "AGENTS.md"):
        text = (ROOT / name).read_text(encoding="utf-8")
        assert "\u2014" not in text, name
        assert "\u2013" not in text, name
