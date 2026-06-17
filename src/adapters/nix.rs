use std::collections::HashMap;
use std::path::Path;

use tree_sitter::{Node, Parser};

use crate::grammars;
use crate::model::{FileMap, Language, LineSpan, ParseError, Symbol, SymbolKind};

pub fn parse(path: &Path, source: String) -> FileMap {
    let mut parser = Parser::new();
    let mut parse_errors = Vec::new();
    let Some(language) = grammars::language(Language::Nix) else {
        parse_errors.push(ParseError {
            line: 1,
            message: "failed to load Nix grammar".to_owned(),
        });
        return file_map(path, source, Vec::new(), parse_errors);
    };

    if let Err(err) = parser.set_language(&language) {
        parse_errors.push(ParseError {
            line: 1,
            message: format!("failed to load Nix grammar: {err}"),
        });
        return file_map(path, source, Vec::new(), parse_errors);
    }

    let Some(tree) = parser.parse(&source, None) else {
        parse_errors.push(ParseError {
            line: 1,
            message: "tree-sitter returned no parse tree".to_owned(),
        });
        return file_map(path, source, Vec::new(), parse_errors);
    };

    let root = tree.root_node();
    collect_parse_errors(root, &mut parse_errors);
    let symbols = Collector::new(&source).collect(root);
    file_map(path, source, symbols, parse_errors)
}

fn file_map(
    path: &Path,
    source: String,
    symbols: Vec<Symbol>,
    parse_errors: Vec<ParseError>,
) -> FileMap {
    let mut file = FileMap::new(path.to_path_buf(), Language::Nix, source, symbols);
    file.parse_errors = parse_errors;
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
        self.collect_container(root, None, &mut symbols);
        symbols
    }

    fn collect_container(
        &mut self,
        node: Node<'_>,
        parent_key: Option<&str>,
        symbols: &mut Vec<Symbol>,
    ) {
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            if child.kind() == "binding" {
                self.push_binding(child, parent_key, symbols);
            } else {
                self.collect_container(child, parent_key, symbols);
            }
        }
    }

    fn push_binding(
        &mut self,
        node: Node<'_>,
        parent_key: Option<&str>,
        symbols: &mut Vec<Symbol>,
    ) {
        let Some(attrpath) = node.child_by_field_name("attrpath") else {
            return;
        };
        let Some(name) = self.attrpath(attrpath) else {
            return;
        };

        let key = parent_key.map_or_else(|| name.clone(), |parent| format!("{parent}.{name}"));
        let key = self.unique_key(key);
        let mut symbol = Symbol::new(
            key.clone(),
            SymbolKind::Field,
            name,
            self.signature(node),
            line_span(node),
        );
        symbol.parent_key = parent_key.map(str::to_owned);

        if let Some(expression) = node.child_by_field_name("expression") {
            self.collect_container(expression, Some(key.as_str()), &mut symbol.children);
        }

        symbols.push(symbol);
    }

    fn attrpath(&self, node: Node<'_>) -> Option<String> {
        let mut parts = Vec::new();
        let mut cursor = node.walk();
        for attr in node.children_by_field_name("attr", &mut cursor) {
            let part = self.node_text(attr)?.trim();
            if part.is_empty() {
                continue;
            }
            parts.push(strip_quotes(part));
        }
        (!parts.is_empty()).then(|| parts.join("."))
    }

    fn node_text(&self, node: Node<'_>) -> Option<&'a str> {
        node.utf8_text(self.source.as_bytes()).ok()
    }

    fn unique_key(&mut self, key: String) -> String {
        let count = self.key_counts.entry(key.clone()).or_insert(0);
        *count += 1;
        if *count == 1 {
            return key;
        }
        format!("{key}#{count}")
    }

    fn signature(&self, node: Node<'_>) -> String {
        self.node_text(node)
            .and_then(|text| text.lines().next())
            .map(str::trim)
            .unwrap_or_default()
            .to_owned()
    }
}

fn strip_quotes(text: &str) -> String {
    let text = text.trim();
    text.strip_prefix('"')
        .and_then(|text| text.strip_suffix('"'))
        .unwrap_or(text)
        .to_owned()
}

fn line_span(node: Node<'_>) -> LineSpan {
    LineSpan::new(node.start_position().row + 1, node.end_position().row + 1)
}
