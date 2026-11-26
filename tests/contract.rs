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

fn commonmark_lines(text: &str) -> Vec<&str> {
    let bytes = text.as_bytes();
    let mut out = Vec::new();
    let mut start = 0;
    let mut idx = 0;
    while idx < bytes.len() {
        match bytes[idx] {
            b'\r' => {
                out.push(&text[start..idx]);
                idx += if bytes.get(idx + 1) == Some(&b'\n') {
                    2
                } else {
                    1
                };
                start = idx;
            }
            b'\n' => {
                out.push(&text[start..idx]);
                idx += 1;
                start = idx;
            }
            _ => idx += 1,
        }
    }
    if start < bytes.len() {
        out.push(&text[start..]);
    }
    out
}

fn prose_typography_violations(label: &str, text: &str) -> Vec<String> {
    let mut violations = Vec::new();
    let mut fence_ticks: Option<usize> = None;
    for (idx, line) in commonmark_lines(text).into_iter().enumerate() {
        let line_no = idx + 1;
        if line.contains('\u{2014}') {
            violations.push(format!("{label}:{line_no} carries an em dash"));
        }
        if line.contains('\u{2013}') {
            violations.push(format!("{label}:{line_no} carries an en dash"));
        }
        let leading_spaces = line.bytes().take_while(|&b| b == b' ').count();
        let rest = &line[leading_spaces..];
        if let Some(open_ticks) = fence_ticks {
            if leading_spaces <= 3 {
                let ticks = rest.bytes().take_while(|&b| b == b'`').count();
                if ticks >= open_ticks && rest[ticks..].bytes().all(|b| b == b' ' || b == b'\t') {
                    fence_ticks = None;
                }
            }
            continue;
        }
        if leading_spaces <= 3 {
            let ticks = rest.bytes().take_while(|&b| b == b'`').count();
            if ticks >= 3 && !rest[ticks..].contains('`') {
                fence_ticks = Some(ticks);
                continue;
            }
        }
        if line.trim().is_empty() {
            continue;
        }
        if leading_spaces <= 3 {
            let hashes = rest.bytes().take_while(|&b| b == b'#').count();
            if (1..=6).contains(&hashes)
                && matches!(rest.as_bytes().get(hashes), None | Some(b' ') | Some(b'\t'))
            {
                continue;
            }
        }
        if line.contains(':') {
            violations.push(format!("{label}:{line_no} carries a prose colon"));
        }
        if line.contains(';') {
            violations.push(format!("{label}:{line_no} carries a prose semicolon"));
        }
    }
    if fence_ticks.is_some() {
        violations.push(format!("{label} carries an unclosed code fence"));
    }
    violations
}

#[test]
fn test_prose_carries_no_forbidden_dashes_or_colons_or_semicolons() {
    // Prove the helper discriminates on synthetic clean and violating texts,
    // including CommonMark ATX heading, code fence, indentation, and EOF rules.
    let clean = "\
# Heading: colons and; semicolons outside prose
   ###### Level six:\tallowed in heading
##

Plain contract prose stands here with no forbidden marks.
   \t  
- TP-1 Bullet prose also stands clean here.

```rust
let allowed: usize = 1;
```

   ````md
```
still_inside_four_tick_fence: allowed;
```
    ````
   ````\t 
";
    assert!(
        prose_typography_violations("synthetic-clean", clean).is_empty(),
        "clean markdown passes"
    );

    let cases = [
        (
            "synthetic-colon",
            "Two corrections stand on counted figures: the word None stands here.\n",
            "synthetic-colon:1 carries a prose colon",
        ),
        (
            "synthetic-bullet-colon",
            "- TP-2 Prose bullet: carries a forbidden colon.\n",
            "synthetic-bullet-colon:1 carries a prose colon",
        ),
        (
            "synthetic-nested-bullet-colon",
            "    - nested: fails\n",
            "synthetic-nested-bullet-colon:1 carries a prose colon",
        ),
        (
            "synthetic-tab-indented-semicolon",
            "\tindented_prose; fails\n",
            "synthetic-tab-indented-semicolon:1 carries a prose semicolon",
        ),
        (
            "synthetic-semicolon",
            "The examination stands unchanged in its readings; what changes is the shell.\n",
            "synthetic-semicolon:1 carries a prose semicolon",
        ),
        (
            "synthetic-after-closed-fence-colon",
            "```\nallowed: 1;\n```\nAfter closed fence: fails.\n",
            "synthetic-after-closed-fence-colon:4 carries a prose colon",
        ),
        (
            "synthetic-after-closed-fence-semicolon",
            "```\nallowed: 1;\n```\nAfter closed fence; fails.\n",
            "synthetic-after-closed-fence-semicolon:4 carries a prose semicolon",
        ),
        (
            "synthetic-four-tick-after-close",
            "````\n```\ninside: allowed;\n````\noutside: fails\n",
            "synthetic-four-tick-after-close:5 carries a prose colon",
        ),
        (
            "synthetic-four-tick-inner-three-unclosed",
            "````\n```\n",
            "synthetic-four-tick-inner-three-unclosed carries an unclosed code fence",
        ),
        (
            "synthetic-four-space-ticks-no-fence",
            "    ```\nfollowing prose: fails.\n",
            "synthetic-four-space-ticks-no-fence:2 carries a prose colon",
        ),
        (
            "synthetic-unclosed-fence",
            "```rust\nlet allowed: usize = 1;\n",
            "synthetic-unclosed-fence carries an unclosed code fence",
        ),
        (
            "synthetic-inline-ticks",
            "```inline```: trailing prose colon still fails.\n",
            "synthetic-inline-ticks:1 carries a prose colon",
        ),
        (
            "synthetic-hash-no-space",
            "#1 not a heading: fails\n",
            "synthetic-hash-no-space:1 carries a prose colon",
        ),
        (
            "synthetic-seven-hashes",
            "####### seven hashes: fails\n",
            "synthetic-seven-hashes:1 carries a prose colon",
        ),
        (
            "synthetic-four-space-hash",
            "    # four spaces before hash: fails\n",
            "synthetic-four-space-hash:1 carries a prose colon",
        ),
        (
            "synthetic-em-dash",
            "A sentence \u{2014} with an em dash fails.\n",
            "synthetic-em-dash:1 carries an em dash",
        ),
        (
            "synthetic-fenced-em-dash",
            "```\n\u{2014}\n```\n",
            "synthetic-fenced-em-dash:2 carries an em dash",
        ),
        (
            "synthetic-en-dash",
            "# Heading 1\u{2013}5 with an en dash fails\n",
            "synthetic-en-dash:1 carries an en dash",
        ),
        (
            "synthetic-lone-cr-after-heading",
            "# Heading\rForbidden: prose\r",
            "synthetic-lone-cr-after-heading:2 carries a prose colon",
        ),
        (
            "synthetic-crlf-after-heading",
            "# Heading\r\nForbidden; prose\r\n",
            "synthetic-crlf-after-heading:2 carries a prose semicolon",
        ),
    ];
    for (name, sample, expected) in cases {
        let found = prose_typography_violations(name, sample);
        assert!(
            found.iter().any(|msg| msg == expected),
            "case {name} must fail with {expected}, got {found:?}"
        );
    }
    assert_eq!(
        prose_typography_violations(
            "synthetic-four-space-ticks-exact",
            "    ```\nfollowing prose: fails.\n"
        ),
        vec!["synthetic-four-space-ticks-exact:2 carries a prose colon"]
    );

    for rel in ["CONTRACT.md", "README.md"] {
        let text = read(rel);
        let found = prose_typography_violations(rel, &text);
        assert!(
            found.is_empty(),
            "{rel} carries typography violations: {found:?}"
        );
    }
    let agents = read("AGENTS.md");
    assert!(!agents.contains('\u{2014}'), "AGENTS.md carries no em dash");
    assert!(!agents.contains('\u{2013}'), "AGENTS.md carries no en dash");
}
