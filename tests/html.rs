use std::path::Path;

use lupa::{parse_source, Language, LineSpan, SymbolKind};

#[test]
fn html_maps_semantic_structure_without_generic_dom_noise() {
    let map = parse_source(
        Path::new("source_shapes.html"),
        Language::Html,
        include_str!("fixtures/source_shapes.html").to_owned(),
    )
    .expect("HTML parsing should succeed");

    assert!(map.parse_errors.is_empty(), "{:?}", map.parse_errors);
    let symbols = map.all_symbols();
    let keys = symbols
        .iter()
        .map(|symbol| symbol.key.as_str())
        .collect::<Vec<_>>();

    for key in [
        "title",
        "style",
        "Audit controls",
        "Audit controls.primary-nav",
        "Repository audit",
        "Repository audit.Findings",
        "Repository audit.Findings.Attribute precedence is visible",
        "Repository audit.Findings.Critical finding",
        "Repository audit.Findings.Critical finding.Ignored article heading",
        "Repository audit.Findings.details",
        "Repository audit.Findings.details.Ignored details heading",
        "Repository audit.Findings.Evidence",
        "Repository audit.Findings.Evidence.Trace",
        "Repository audit.Findings.section",
        "Repository audit.Alpha.Summary",
        "Repository audit.Alpha.Summary#2",
        "Repository audit.Beta.Summary",
        "Repository audit.static-id",
        "footer",
        "script",
        "script#2",
    ] {
        assert!(keys.contains(&key), "missing HTML key {key:?}: {keys:?}");
    }

    for noise in ["html", "head", "body", "div", "p", "a"] {
        assert!(!keys.contains(&noise), "generic DOM node leaked: {noise}");
    }
}

#[test]
fn html_labels_have_stable_kinds_ranges_and_duplicate_scope() {
    let map = parse_source(
        Path::new("source_shapes.html"),
        Language::Html,
        include_str!("fixtures/source_shapes.html").to_owned(),
    )
    .expect("HTML parsing should succeed");
    let symbols = map.all_symbols();

    let findings = symbols
        .iter()
        .find(|symbol| symbol.key == "Repository audit.Findings")
        .expect("data-screen-label should win");
    assert_eq!(findings.kind, SymbolKind::Node);
    assert_eq!(findings.name, "Findings");
    assert_eq!(findings.range, LineSpan::new(16, 28));
    assert_eq!(
        findings.signature,
        "<SECTION DATA-SCREEN-LABEL=\"Findings\" aria-label=\"Ignored label\" id=\"ignored-id\">"
    );

    let heading = symbols
        .iter()
        .find(|symbol| symbol.key == "Repository audit.Findings.Attribute precedence is visible")
        .expect("attribute-labelled section should retain its heading");
    assert_eq!(heading.kind, SymbolKind::Heading);
    assert_eq!(heading.range, LineSpan::new(17, 17));

    let alpha_summary = symbols
        .iter()
        .find(|symbol| symbol.key == "Repository audit.Alpha.Summary")
        .expect("first Alpha summary");
    let beta_summary = symbols
        .iter()
        .find(|symbol| symbol.key == "Repository audit.Beta.Summary")
        .expect("Beta summary");
    assert_eq!(
        alpha_summary.parent_key.as_deref(),
        Some("Repository audit.Alpha")
    );
    assert_eq!(
        beta_summary.parent_key.as_deref(),
        Some("Repository audit.Beta")
    );
    assert!(symbols
        .iter()
        .any(|symbol| symbol.key == "Repository audit.Alpha.Summary#2"));
    assert!(!symbols
        .iter()
        .any(|symbol| symbol.key == "Repository audit.Beta.Summary#2"));

    let title = symbols
        .iter()
        .find(|symbol| symbol.key == "title")
        .expect("title symbol");
    assert_eq!(title.kind, SymbolKind::Node);
    let script = symbols
        .iter()
        .find(|symbol| symbol.key == "script")
        .expect("script symbol");
    assert_eq!(script.kind, SymbolKind::Node);
}

#[test]
fn html_rejects_template_expressions_as_labels() {
    let map = parse_source(
        Path::new("template.html"),
        Language::Html,
        include_str!("fixtures/source_shapes.html").to_owned(),
    )
    .expect("HTML template parsing should succeed");
    let symbols = map.all_symbols();

    assert!(symbols
        .iter()
        .any(|symbol| symbol.key == "Repository audit.static-id"));
    for symbol in symbols {
        assert!(!symbol.key.contains("{{"), "dynamic key: {}", symbol.key);
        assert!(
            !symbol.key.contains("dynamic_heading"),
            "dynamic key: {}",
            symbol.key
        );
    }
}

#[test]
fn malformed_html_reports_a_parse_error() {
    let map = parse_source(
        Path::new("broken.html"),
        Language::Html,
        "<main><section id=\"broken><h2>Broken</section></main>".to_owned(),
    )
    .expect("malformed HTML should return a partial map");

    assert!(!map.parse_errors.is_empty());
}
