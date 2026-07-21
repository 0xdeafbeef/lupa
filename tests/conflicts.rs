use std::path::{Path, PathBuf};

use lupa::model::{FileMap, ParseError};
use lupa::{context, parse_source, render, Language, LineSpan};

fn parse(source: &str) -> FileMap {
    parse_source(Path::new("input.rs"), Language::Rust, source.to_owned()).unwrap()
}

fn regions(source: &str) -> Vec<LineSpan> {
    parse(source).conflict_regions
}

#[test]
fn detects_git_and_jj_marker_families() {
    let cases = [
        (
            "<<<<<<< left\nleft\n=======\nright\n>>>>>>> right\n",
            LineSpan::new(1, 5),
        ),
        (
            "<<<<<<< left\nleft\n||||||| base\nbase\n=======\nright\n>>>>>>> right\n",
            LineSpan::new(1, 7),
        ),
        (
            "<<<<<<< Conflict\n%%%%%%% diff\n-old\n+new\n+++++++ side\nnewer\n>>>>>>> end\n",
            LineSpan::new(1, 7),
        ),
        (
            "<<<<<<< Conflict\n%%%%%%% diff\n\\\\\\\\\\\\\\ note\n-old\n+new\n+++++++ side\nnewer\n>>>>>>> end\n",
            LineSpan::new(1, 8),
        ),
        (
            "<<<<<<< Conflict\n+++++++ snapshot\nsnapshot\n%%%%%%% diff\n-old\n+new\n>>>>>>> end\n",
            LineSpan::new(1, 7),
        ),
        (
            "<<<<<<< Conflict\n+++++++ side 1\none\n------- base\nbase\n+++++++ side 2\ntwo\n>>>>>>> end\n",
            LineSpan::new(1, 8),
        ),
        (
            "<<<<<<< Conflict\n+++++++ side 1\none\n------- base 1\nbase\n+++++++ side 2\ntwo\n------- base 2\nbase\n+++++++ side 3\nthree\n>>>>>>> end\n",
            LineSpan::new(1, 12),
        ),
    ];

    for (source, expected) in cases {
        assert_eq!(regions(source), vec![expected], "source:\n{source}");
    }
}

#[test]
fn accepts_width_line_endings_empty_sides_and_eof() {
    let wide = "<<<<<<<< left\n========\n>>>>>>>> right";
    assert_eq!(regions(wide), vec![LineSpan::new(1, 3)]);

    let crlf = "before\r\n<<<<<<<\r\n=======\r\n>>>>>>>\r\nafter\r\n";
    assert_eq!(regions(crlf), vec![LineSpan::new(2, 4)]);

    let eof = "<<<<<<<\nleft\n=======\nright\n>>>>>>>";
    assert_eq!(regions(eof), vec![LineSpan::new(1, 5)]);
}

#[test]
fn detects_multiple_and_adjacent_regions_in_source_order() {
    let source = concat!(
        "head\n",
        "<<<<<<<\n=======\n>>>>>>>\n",
        "<<<<<<<\n%%%%%%%\n-old\n+new\n+++++++\nright\n>>>>>>>\n",
    );
    assert_eq!(regions(source), vec![
        LineSpan::new(2, 4),
        LineSpan::new(5, 11)
    ]);
}

#[test]
fn rejects_malformed_markers_bodies_and_false_positives() {
    let invalid = [
        "<<<<<<\n=======\n>>>>>>\n",
        " <<<<<<<\n=======\n>>>>>>>\n",
        "<<<<<<<joined\n=======\n>>>>>>>\n",
        "<<<<<<<\n======= label\n>>>>>>>\n",
        "<<<<<<<\nleft\n>>>>>>>\n",
        "<<<<<<<\n========\n>>>>>>>\n",
        "<<<<<<<\n||||||| base\n||||||| duplicate\n=======\n>>>>>>>\n",
        "<<<<<<<\n=======\n||||||| late\n>>>>>>>\n",
        "<<<<<<<\n=======\n=======\n>>>>>>>\n",
        "<<<<<<<\n%%%%%%% diff\nbad-prefix\n+++++++ side\n>>>>>>>\n",
        "<<<<<<<\n%%%%%%% diff\n>>>>>>>\n",
        "<<<<<<<\n%%%%%%% diff\n\\\\\\\\\\\\\\ note\n\\\\\\\\\\\\\\ duplicate\n+++++++ side\n>>>>>>>\n",
        ">>>>>>> stray\n",
        "<<<<<<< incomplete\n",
        "<<<<<<<\n=======\n>>>>>>>> mismatched\n>>>>>>>\n",
    ];

    for source in invalid {
        assert!(regions(source).is_empty(), "source:\n{source}");
    }
}

#[test]
fn later_start_recovers_after_unmatched_opener() {
    let source = concat!(
        "<<<<<<< abandoned\n",
        "ordinary\n",
        "<<<<<<< recovered\n",
        "left\n",
        "=======\n",
        "right\n",
        ">>>>>>> end\n",
    );
    assert_eq!(regions(source), vec![LineSpan::new(3, 7)]);
}

#[test]
fn git_side_content_may_resemble_jj_controls() {
    let source = concat!(
        "<<<<<<< left\n",
        "+++++++ jj-looking content\n",
        "left\n",
        "=======\n",
        "right\n",
        ">>>>>>> end\n",
    );
    assert_eq!(regions(source), vec![LineSpan::new(1, 6)]);
}

#[test]
fn preserves_original_source_and_parser_output() {
    let source = include_str!("fixtures/conflicted.rs");
    let file = parse(source);

    assert_eq!(file.source, source);
    assert_eq!(file.conflict_regions, vec![LineSpan::new(5, 11)]);
    let keys = file
        .all_symbols()
        .into_iter()
        .map(|symbol| symbol.key.as_str())
        .collect::<Vec<_>>();
    assert!(keys.contains(&"before"), "keys: {keys:?}");
    assert!(keys.contains(&"after"), "keys: {keys:?}");
}

#[test]
fn diagnostics_follow_parser_output_and_precede_context_hits() {
    let mut file = parse("<<<<<<<\n=======\n>>>>>>>\n");
    file.parse_errors = vec![ParseError {
        line: 2,
        message: "parser detail".to_owned(),
    }];
    file.warnings = vec!["adapter detail".to_owned()];
    let expected = concat!(
        "# warning: parse error at L2: parser detail\n",
        "# warning: adapter detail\n",
        "# warning: unresolved merge conflict at L1-L3\n",
    );

    let mut map = String::new();
    render::render_map(&file, &mut map).unwrap();
    assert!(map.contains(expected), "map:\n{map}");

    let mut context_out = String::new();
    context::render_context(&file, &[1], &mut context_out).unwrap();
    assert!(context_out.starts_with(expected), "context:\n{context_out}");
    assert!(
        context_out.contains("input.rs no-symbol hits L1\n"),
        "context:\n{context_out}"
    );
}

#[test]
fn digest_renders_only_nonzero_conflict_counts() {
    let one = parse("<<<<<<<\n=======\n>>>>>>>\n");
    let two = parse("<<<<<<<\n=======\n>>>>>>>\n<<<<<<<\n=======\n>>>>>>>\n");
    let clean = parse("fn clean() {}\n");
    let mut out = String::new();

    render::render_digest(&[one, two, clean], &mut out).unwrap();

    let lines = out.lines().collect::<Vec<_>>();
    assert!(lines[0].contains(" 1C"), "digest:\n{out}");
    assert!(lines[1].contains(" 2C"), "digest:\n{out}");
    assert!(!lines[2].contains('C'), "digest:\n{out}");
}

#[test]
fn clean_renderers_remain_byte_for_byte_compatible() {
    let file = parse_source(
        Path::new("clean.rs"),
        Language::Rust,
        "fn clean() {}\n".to_owned(),
    )
    .unwrap();

    let mut map = String::new();
    render::render_map(&file, &mut map).unwrap();
    assert_eq!(map, "# clean.rs [rust] 1L 14B 1S\nL1 clean fn clean()\n");

    let mut digest = String::new();
    render::render_digest(std::slice::from_ref(&file), &mut digest).unwrap();
    assert_eq!(digest, "clean.rs [rust] 1L 1S clean@L1\n");

    let mut keys = String::new();
    render::render_keys(&file, &mut keys).unwrap();
    assert_eq!(keys, "clean L1\n");

    let mut show = String::new();
    render::render_show(&file, &["clean".to_owned()], &mut show).unwrap();
    assert_eq!(show, "# clean@L1\nfn clean() {}\n");

    let mut context_out = String::new();
    context::render_context(&file, &[1], &mut context_out).unwrap();
    assert_eq!(context_out, "clean.rs clean@L1 hits L1 fn clean()\n");
}

#[test]
fn repeated_detection_and_rendering_is_identical() {
    let source = "<<<<<<<\n=======\n>>>>>>>\n";
    let first = parse(source);
    let second = parse(source);
    assert_eq!(first.conflict_regions, second.conflict_regions);

    let mut first_out = String::new();
    let mut second_out = String::new();
    render::render_map(&first, &mut first_out).unwrap();
    render::render_map(&second, &mut second_out).unwrap();
    assert_eq!(first_out, second_out);
}

#[test]
fn file_map_new_starts_without_derived_regions() {
    let file = FileMap::new(
        PathBuf::from("input.rs"),
        Language::Rust,
        "<<<<<<<\n=======\n>>>>>>>\n".to_owned(),
        Vec::new(),
    );
    assert!(file.conflict_regions.is_empty());
}
