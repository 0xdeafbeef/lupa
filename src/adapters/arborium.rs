use std::collections::HashMap;
use std::path::Path;

use ::arborium::tree_sitter::{Node, Parser};

use crate::model::{FileMap, Language, LineSpan, ParseError, Symbol, SymbolKind};

const LIMITED_FALLBACK_WARNING: &str = "limited fallback adapter: top-level syntax nodes only";

pub fn parse(path: &Path, language: Language, source: String) -> FileMap {
    let parser_name = language
        .arborium_parser_name()
        .expect("fallback adapter called with non-fallback language");
    let mut parser = Parser::new();
    let mut parse_errors = Vec::new();
    let Some(grammar) = ::arborium::get_language(parser_name) else {
        parse_errors.push(ParseError {
            line: 1,
            message: format!(
                "failed to load {language} grammar: Arborium grammar '{parser_name}' is not enabled"
            ),
        });
        return file_map(path, language, source, Vec::new(), parse_errors);
    };

    if let Err(err) = parser.set_language(&grammar) {
        parse_errors.push(ParseError {
            line: 1,
            message: format!("failed to load {language} grammar: {err}"),
        });
        return file_map(path, language, source, Vec::new(), parse_errors);
    }

    let Some(tree) = parser.parse(&source, None) else {
        parse_errors.push(ParseError {
            line: 1,
            message: "tree-sitter returned no parse tree".to_owned(),
        });
        return file_map(path, language, source, Vec::new(), parse_errors);
    };

    let root = tree.root_node();
    collect_parse_errors(root, &mut parse_errors);
    parse_errors.dedup_by(|left, right| left.line == right.line && left.message == right.message);
    let symbols = Collector::new(&source).collect(root);
    file_map(path, language, source, symbols, parse_errors)
}

fn file_map(
    path: &Path,
    language: Language,
    source: String,
    symbols: Vec<Symbol>,
    parse_errors: Vec<ParseError>,
) -> FileMap {
    let mut file = FileMap::new(path.to_path_buf(), language, source, symbols);
    file.parse_errors = parse_errors;
    file.warnings.push(LIMITED_FALLBACK_WARNING.to_owned());
    file
}

fn collect_parse_errors(node: Node<'_>, parse_errors: &mut Vec<ParseError>) {
    if node.is_error() || node.is_missing() {
        parse_errors.push(ParseError {
            line: line_span(node).start_line,
            message: format!("parse error in {}", node.kind()),
        });
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        if child.has_error() || child.is_missing() {
            collect_parse_errors(child, parse_errors);
        }
    }
}

struct Collector<'a> {
    source: &'a str,
    key_counts: HashMap<String, usize>,
}

impl<'a> Collector<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            key_counts: HashMap::new(),
        }
    }

    fn collect(&mut self, root: Node<'_>) -> Vec<Symbol> {
        let mut symbols = Vec::new();
        let mut cursor = root.walk();
        for child in root.named_children(&mut cursor) {
            if matches!(child.kind(), "comment" | "line_comment" | "block_comment") {
                continue;
            }
            let name = child.kind().to_owned();
            let key = self.unique_key(name.clone());
            symbols.push(Symbol::new(
                key,
                SymbolKind::Node,
                name,
                self.signature(child),
                line_span(child),
            ));
        }
        symbols
    }

    fn unique_key(&mut self, key: String) -> String {
        let count = self.key_counts.entry(key.clone()).or_default();
        *count += 1;
        if *count == 1 {
            key
        } else {
            format!("{key}#{count}")
        }
    }

    fn signature(&self, node: Node<'_>) -> String {
        let signature = node
            .utf8_text(self.source.as_bytes())
            .ok()
            .and_then(|text| text.lines().next())
            .map(|line| line.split_whitespace().collect::<Vec<_>>().join(" "))
            .unwrap_or_default();
        if signature.is_empty() {
            node.kind().to_owned()
        } else {
            signature
        }
    }
}

fn line_span(node: Node<'_>) -> LineSpan {
    let start_line = node.start_position().row + 1;
    let mut end_line = node.end_position().row + 1;
    if node.end_position().column == 0 && end_line > start_line {
        end_line -= 1;
    }
    LineSpan::new(start_line, end_line)
}
