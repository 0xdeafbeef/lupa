use std::path::Path;

use lupa::{parse_source, render, Language};

#[test]
fn show_suggests_exact_leaf_candidate_for_extra_qualified_key() {
    let source = r"
fn zero_rtt_incoming_buffer_size() -> usize {
    42
}
";
    let file = parse_source(Path::new("-"), Language::Rust, source.to_owned()).unwrap();
    let keys = vec!["tests.zero_rtt_incoming_buffer_size".to_owned()];
    let mut out = String::new();

    render::render_show(&file, &keys, &mut out).unwrap();

    assert!(out.contains("# no tests.zero_rtt_incoming_buffer_size\n"));
    assert!(out.contains("# candidates\n"));
    assert!(out.contains("# zero_rtt_incoming_buffer_size@"));
    assert!(!out.contains("    42\n"));
}
