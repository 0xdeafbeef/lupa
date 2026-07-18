use std::path::Path;

use lupa::{parse_source, Language, LineSpan, SymbolKind};

#[test]
fn ini_duplicate_sections_keep_final_parent_keys_and_ranges() {
    let map = parse_source(
        Path::new("config.ini"),
        Language::Ini,
        include_str!("fixtures/source_shapes.ini").to_owned(),
    )
    .expect("INI parsing should succeed");

    assert!(map.parse_errors.is_empty(), "{:?}", map.parse_errors);
    let first = map
        .symbols
        .iter()
        .find(|symbol| symbol.key == "remote \"origin\"")
        .expect("first remote section");
    assert_eq!(first.kind, SymbolKind::Heading);
    assert_eq!(first.range, LineSpan::new(7, 8));
    assert_eq!(first.children[0].key, "remote \"origin\".url");
    assert_eq!(
        first.children[0].parent_key.as_deref(),
        Some(first.key.as_str())
    );

    let duplicate = map
        .symbols
        .iter()
        .find(|symbol| symbol.key == "remote \"origin\"#2")
        .expect("duplicate remote section");
    assert_eq!(duplicate.range, LineSpan::new(10, 12));
    assert_eq!(duplicate.children[0].key, "remote \"origin\"#2.url");
    assert_eq!(
        duplicate.children[0].parent_key.as_deref(),
        Some(duplicate.key.as_str())
    );
}

#[test]
fn git_diff_blocks_end_before_the_next_file() {
    let map = parse_source(
        Path::new("security.patch"),
        Language::Diff,
        include_str!("fixtures/stress/security.patch").to_owned(),
    )
    .expect("diff parsing should succeed");

    assert!(map.parse_errors.is_empty(), "{:?}", map.parse_errors);
    assert_eq!(map.symbols.len(), 2);
    assert_eq!(map.symbols[0].key, "src/check.rs");
    assert_eq!(map.symbols[0].range, LineSpan::new(1, 13));
    assert_eq!(map.symbols[0].children[0].range, LineSpan::new(5, 13));
    assert_eq!(map.symbols[1].key, "tests/check.rs");
    assert_eq!(map.symbols[1].range, LineSpan::new(14, 25));
    assert_eq!(map.symbols[1].children[0].range, LineSpan::new(19, 25));
}

#[test]
fn loose_diff_groups_multiple_hunks_and_deleted_files() {
    let map = parse_source(
        Path::new("source_shapes.diff"),
        Language::Diff,
        include_str!("fixtures/source_shapes.diff").to_owned(),
    )
    .expect("diff parsing should succeed");

    assert!(map.parse_errors.is_empty(), "{:?}", map.parse_errors);
    assert_eq!(map.symbols.len(), 2);

    let changed = &map.symbols[0];
    assert_eq!(changed.key, "src/lib.rs");
    assert_eq!(changed.range, LineSpan::new(1, 9));
    assert_eq!(changed.children.len(), 2);
    assert_eq!(changed.children[0].key, "src/lib.rs.hunk");
    assert_eq!(changed.children[0].range, LineSpan::new(3, 6));
    assert_eq!(changed.children[1].key, "src/lib.rs.hunk#2");
    assert_eq!(changed.children[1].range, LineSpan::new(7, 9));

    let deleted = &map.symbols[1];
    assert_eq!(deleted.key, "src/old.rs");
    assert_eq!(deleted.signature, "+++ /dev/null");
    assert_eq!(deleted.range, LineSpan::new(10, 13));
    assert_eq!(
        deleted.children[0].parent_key.as_deref(),
        Some("src/old.rs")
    );
}

#[test]
fn malformed_ini_reports_a_parse_error() {
    let map = parse_source(
        Path::new("broken.ini"),
        Language::Ini,
        "[server\nport = 8080\n".to_owned(),
    )
    .expect("INI parsing should return a partial map");

    assert!(!map.parse_errors.is_empty());
}
