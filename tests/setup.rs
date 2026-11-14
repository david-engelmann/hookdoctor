//! The seed pins of obligation PORT2-5, re-expressing the
//! landed Python setup pins as Rust tests over the tree itself.
//! The count sentence stands alone in its own test here, so the
//! closure moves the README line and this pin in one commit, and
//! the gate sentence is asserted at its new wording, the annex
//! moving in the same commit as this suite.

fn read(rel: &str) -> String {
    let path = format!("{}/{}", env!("CARGO_MANIFEST_DIR"), rel);
    std::fs::read_to_string(&path).unwrap_or_else(|err| panic!("{rel} reads: {err}"))
}

#[test]
fn test_seed_files_present() {
    let readme = read("README.md");
    assert!(readme.contains("# hookdoctor"));
    assert!(readme.contains("It examines the hooks configured in an agent harness"));
    assert!(readme.contains("## Licence"));
    let license = read("LICENSE");
    assert!(license.contains("MIT License"));
    let gitignore = read(".gitignore");
    assert!(gitignore.contains("__pycache__/"));
    assert!(gitignore.contains(".env"));
}

#[test]
fn test_readme_records_the_closed_round_count() {
    let readme = read("README.md");
    assert!(
        readme.contains("Closed rounds stand at 4 of 100"),
        "the README records the closed round count"
    );
}

#[test]
fn test_agents_template_and_annex() {
    let agents = read("AGENTS.md");
    assert!(agents.starts_with("# AGENTS.md"));
    assert!(agents.contains("39 repo storefront perfection program"));
    let marker = "<!-- repo annex below this line -->";
    assert_eq!(agents.matches(marker).count(), 1);
    let annex = agents.split(marker).nth(1).expect("the annex stands");
    assert!(annex.contains("Hookdoctor audits the harness's hooks"));
    assert!(annex.contains("blindgate audits CI's checks"));
    assert!(annex.contains("ingested as local files with synthetic fixtures only"));
    assert!(annex.contains("no network in the tool or the tests"));
    assert!(annex.contains("no real session data may ever become a fixture"));
    assert!(annex.contains("The local gate is cargo test"));
    assert!(annex.contains("closed round count stands in the README and moves only at a closure"));
}
