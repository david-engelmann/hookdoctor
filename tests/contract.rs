//! The contract checkers of obligation PORT2-5. Every name the
//! contract states stands disposed in the name map at
//! `tests/contract_name_map.txt`, either as a Rust test in this
//! crate under its own name, or as a recorded case standing in
//! one of the conformance recordings, or as a recorded section
//! standing there. The map carries no name the contract does not
//! state. The public files carry no em dash and no en dash.

use std::collections::BTreeSet;

use hookdoctor::jsonvalue::JsonValue;
use hookdoctor::reader::read_json_document;

fn read(rel: &str) -> String {
    let path = format!("{}/{}", env!("CARGO_MANIFEST_DIR"), rel);
    std::fs::read_to_string(&path).unwrap_or_else(|err| panic!("{rel} reads: {err}"))
}

fn contract_names() -> BTreeSet<String> {
    let contract = read("CONTRACT.md");
    let mut names = BTreeSet::new();
    for token in
        contract.split(|ch: char| !(ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_'))
    {
        if token.len() > 5 && token.starts_with("test_") {
            names.insert(token.to_string());
        }
    }
    names
}

fn recording_document(rel: &str) -> JsonValue {
    read_json_document(&read(rel), "recordings").expect("the recordings parse")
}

/// The case lists a recordings document carries, whether it
/// stands as one flat list of cases under its cases key or as
/// named sections each standing as a case list of their own.
fn case_lists(document: &JsonValue) -> Vec<&Vec<JsonValue>> {
    let mut lists = Vec::new();
    if let Some(cases) = document.get("cases").and_then(|value| value.as_list()) {
        lists.push(cases);
    }
    if let Some(pairs) = document.as_object() {
        for (_key, value) in pairs {
            match value.as_list() {
                Some(cases) => lists.push(cases),
                None => {
                    if let Some(cases) = value.get("cases").and_then(|inner| inner.as_list()) {
                        lists.push(cases);
                    }
                }
            }
        }
    }
    lists
}

fn recording_case_stands(rel: &str, target: &str) -> bool {
    let document = recording_document(rel);
    case_lists(&document).iter().any(|cases| {
        cases
            .iter()
            .any(|case| case.get("name").and_then(|value| value.as_text()) == Some(target))
    })
}

fn recording_section_stands(rel: &str, target: &str) -> bool {
    let document = recording_document(rel);
    if let Some(section) = document.get(target) {
        if let Some(cases) = section.as_list() {
            if !cases.is_empty() {
                return true;
            }
        }
        if let Some(cases) = section.get("cases").and_then(|value| value.as_list()) {
            if !cases.is_empty() {
                return true;
            }
        }
    }
    case_lists(&document).iter().any(|cases| {
        cases.iter().any(|case| {
            case.get("section").and_then(|value| value.as_text()) == Some(target)
                || case.get("kind").and_then(|value| value.as_text()) == Some(target)
        })
    })
}

fn rust_test_stands(rel: &str, name: &str) -> bool {
    read(rel).contains(&format!("fn {name}("))
}

#[test]
fn test_every_contract_name_is_disposed() {
    let names = contract_names();
    assert!(
        names.len() >= 71,
        "the contract states its proof names, found {}",
        names.len()
    );
    let map = read("tests/contract_name_map.txt");
    let mut disposed: BTreeSet<String> = BTreeSet::new();
    for line in map.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (name, rest) = line
            .split_once(" => ")
            .unwrap_or_else(|| panic!("the map line {line} carries its arrow"));
        let mut parts = rest.splitn(3, ' ');
        let kind = parts.next().expect("a map line carries its kind");
        let file = parts.next().expect("a map line carries its file");
        let target = parts.next().unwrap_or("");
        let stands = match kind {
            "rust-test" => rust_test_stands(file, name),
            "recording-case" => recording_case_stands(file, target),
            "recording-section" => recording_section_stands(file, target),
            other => panic!("the map line {line} carries an unknown kind {other}"),
        };
        assert!(
            stands,
            "the contract name {name} is disposed as {kind} {file} {target} and it does not stand"
        );
        disposed.insert(name.to_string());
    }
    for name in &names {
        assert!(
            disposed.contains(name),
            "the contract name {name} has no disposition in the map"
        );
    }
    for name in &disposed {
        assert!(
            names.contains(name),
            "the map disposes {name}, which the contract does not state"
        );
    }
}

#[test]
fn test_contract_has_no_forbidden_dashes() {
    for rel in ["CONTRACT.md", "README.md", "AGENTS.md"] {
        let text = read(rel);
        assert!(!text.contains('\u{2014}'), "{rel} carries no em dash");
        assert!(!text.contains('\u{2013}'), "{rel} carries no en dash");
    }
}
