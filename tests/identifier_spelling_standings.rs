//! The pins of slice 43, the hook identifier
//! spelling standings, over settings documents
//! the pins write as literals and hand to the
//! public configuration reading. HD2-1 names
//! the identifier in general terms as built
//! from the event and the group position and
//! the entry position, and the pins below state
//! that spelling at its fine grain, the colon
//! at both joins and the positions counting
//! from zero and the plain decimal form and
//! the event standing verbatim and position
//! alone parting identical entries. Every pin
//! stands as a fence on the landed code, this
//! slice moving no production code.

use hookdoctor::ingest::{parse_config, HookConfig};
use hookdoctor::reader::read_json_document;

fn config(text: &str) -> Result<Vec<HookConfig>, String> {
    let document =
        read_json_document(text, "configuration").map_err(|error| error.message().to_string())?;
    parse_config(&document).map_err(|error| error.message().to_string())
}

fn ids(text: &str) -> Vec<String> {
    config(text)
        .expect("the reading yields")
        .iter()
        .map(|hook| hook.id.clone())
        .collect()
}

fn entry(command: &str) -> String {
    format!("{{\"command\": \"{command}\"}}")
}

fn group(entries: &[String]) -> String {
    format!("{{\"hooks\": [{}]}}", entries.join(", "))
}

fn doc(event: &str, groups: &[String]) -> String {
    format!("{{\"hooks\": {{\"{event}\": [{}]}}}}", groups.join(", "))
}

#[test]
fn test_the_identifier_spells_the_event_and_the_two_positions_joined_by_colons() {
    let single = doc("PreToolUse", &[group(&[entry("a.sh")])]);
    assert_eq!(ids(&single), ["PreToolUse:0:0"]);
    let walked = "{\"hooks\": {\"A\": [{\"hooks\": [{\"command\": \"a\"}, {\"command\": \"b\"}]}, {\"hooks\": [{\"command\": \"c\"}]}], \"B\": [{\"hooks\": [{\"command\": \"d\"}]}]}}";
    assert_eq!(ids(walked), ["A:0:0", "A:0:1", "A:1:0", "B:0:0"]);
}

#[test]
fn test_an_empty_group_occupies_its_group_position_in_the_identifier() {
    let standing = "{\"hooks\": {\"Stop\": [{\"hooks\": []}, {\"hooks\": [{\"command\": \"a\"}, {\"command\": \"b\"}]}]}}";
    assert_eq!(ids(standing), ["Stop:1:0", "Stop:1:1"]);
    let between = doc(
        "E",
        &[group(&[entry("a.sh")]), group(&[]), group(&[entry("b.sh")])],
    );
    assert_eq!(ids(&between), ["E:0:0", "E:2:0"]);
}

#[test]
fn test_positions_past_nine_spell_in_plain_decimal_without_padding() {
    let mut groups: Vec<String> = (0..10).map(|_| group(&[])).collect();
    groups.push(group(&[entry("a.sh")]));
    let by_group = doc("E", &groups);
    assert_eq!(ids(&by_group), ["E:10:0"]);
    let entries: Vec<String> = (0..11).map(|i| entry(&format!("e{i}.sh"))).collect();
    let by_entry = doc("E", &[group(&entries)]);
    let built = ids(&by_entry);
    assert_eq!(built.len(), 11);
    assert_eq!(built[9], "E:0:9");
    assert_eq!(built[10], "E:0:10");
}

#[test]
fn test_the_event_stands_verbatim_in_the_identifier_at_its_boundary_shapes() {
    assert_eq!(ids(&doc("", &[group(&[entry("a.sh")])])), [":0:0"]);
    assert_eq!(ids(&doc("A:B", &[group(&[entry("a.sh")])])), ["A:B:0:0"]);
    assert_eq!(
        ids(&doc("My Event", &[group(&[entry("a.sh")])])),
        ["My Event:0:0"]
    );
    assert_eq!(
        ids(&doc("E:0:0", &[group(&[entry("a.sh")])])),
        ["E:0:0:0:0"]
    );
}

#[test]
fn test_position_alone_parts_identical_entries_and_no_refused_document_builds_one() {
    let mixed = "{\"hooks\": {\"E\": [{\"matcher\": \"Bash\", \"hooks\": [{\"command\": \"same.sh\"}, {\"command\": \"same.sh\"}]}, {\"matcher\": \"Edit\", \"hooks\": [{\"command\": \"same.sh\"}]}]}}";
    let built = ids(mixed);
    assert_eq!(built, ["E:0:0", "E:0:1", "E:1:0"]);
    let mut sorted = built.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(
        sorted.len(),
        built.len(),
        "identifiers stand pairwise distinct"
    );
    let refused =
        "{\"hooks\": {\"E\": [{\"hooks\": [{\"command\": \"a\"}]}, {\"matcher\": \"x\"}]}}";
    assert_eq!(
        config(refused).expect_err("the reading refuses"),
        "a matcher group carries no hooks list"
    );
}
