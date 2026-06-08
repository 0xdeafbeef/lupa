use std::collections::HashMap;
use std::path::Path;

use tree_sitter::{Node, Parser};

use crate::model::{FileMap, Language, LineSpan, ParseError, Symbol, SymbolKind};

pub fn parse_json(path: &Path, source: String) -> FileMap {
    let mut parser = Parser::new();
    let language = tree_sitter_json::LANGUAGE.into();
    let mut parse_errors = Vec::new();

    if let Err(err) = parser.set_language(&language) {
        parse_errors.push(ParseError {
            line: 1,
            message: format!("failed to load JSON grammar: {err}"),
        });
        return file_map(path, Language::Json, source, Vec::new(), parse_errors);
    }

    let Some(tree) = parser.parse(&source, None) else {
        parse_errors.push(ParseError {
            line: 1,
            message: "tree-sitter returned no parse tree".to_owned(),
        });
        return file_map(path, Language::Json, source, Vec::new(), parse_errors);
    };

    let root = tree.root_node();
    collect_parse_errors(root, &mut parse_errors);
    let symbols = Collector::new(&source).collect_json(root);
    file_map(path, Language::Json, source, symbols, parse_errors)
}

pub fn parse_toml(path: &Path, source: String) -> FileMap {
    let mut parser = Parser::new();
    let language = tree_sitter_toml_ng::LANGUAGE.into();
    let mut parse_errors = Vec::new();

    if let Err(err) = parser.set_language(&language) {
        parse_errors.push(ParseError {
            line: 1,
            message: format!("failed to load TOML grammar: {err}"),
        });
        return file_map(path, Language::Toml, source, Vec::new(), parse_errors);
    }

    let Some(tree) = parser.parse(&source, None) else {
        parse_errors.push(ParseError {
            line: 1,
            message: "tree-sitter returned no parse tree".to_owned(),
        });
        return file_map(path, Language::Toml, source, Vec::new(), parse_errors);
    };

    let root = tree.root_node();
    collect_parse_errors(root, &mut parse_errors);
    let symbols = Collector::new(&source).collect_toml(root);
    file_map(path, Language::Toml, source, symbols, parse_errors)
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

    fn collect_json(&mut self, root: Node<'_>) -> Vec<Symbol> {
        let mut symbols = Vec::new();
        let mut cursor = root.walk();
        for child in root.named_children(&mut cursor) {
            self.collect_json_value(child, None, &mut symbols);
        }
        symbols
    }

    fn collect_json_value(
        &mut self,
        node: Node<'_>,
        parent_key: Option<&str>,
        symbols: &mut Vec<Symbol>,
    ) {
        match node.kind() {
            "object" => {
                let mut cursor = node.walk();
                for child in node.named_children(&mut cursor) {
                    if child.kind() == "pair" {
                        self.push_json_pair(child, parent_key, symbols);
                    }
                }
            }
            "array" => {
                let mut cursor = node.walk();
                for child in node.named_children(&mut cursor) {
                    self.collect_json_value(child, parent_key, symbols);
                }
            }
            _ => {}
        }
    }

    fn push_json_pair(
        &mut self,
        node: Node<'_>,
        parent_key: Option<&str>,
        symbols: &mut Vec<Symbol>,
    ) {
        let Some(key_node) = node.child_by_field_name("key") else {
            return;
        };
        let Some(name) = self.node_text(key_node).map(strip_quotes) else {
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

        if let Some(value) = node.child_by_field_name("value") {
            self.collect_json_value(value, Some(key.as_str()), &mut symbol.children);
        }

        symbols.push(symbol);
    }

    fn collect_toml(&mut self, root: Node<'_>) -> Vec<Symbol> {
        let mut symbols = Vec::new();
        let mut cursor = root.walk();
        for child in root.named_children(&mut cursor) {
            match child.kind() {
                "pair" => self.push_toml_pair(child, None, &mut symbols),
                "table" | "table_array_element" => self.push_toml_table(child, &mut symbols),
                _ => {}
            }
        }
        symbols
    }

    fn push_toml_table(&mut self, node: Node<'_>, symbols: &mut Vec<Symbol>) {
        let Some(key_node) = first_key_child(node) else {
            return;
        };
        let Some(name) = self.toml_key(key_node) else {
            return;
        };

        let key = self.unique_key(name.clone());
        let mut symbol = Symbol::new(
            key.clone(),
            SymbolKind::Heading,
            name,
            self.signature(node),
            toml_table_span(node),
        );

        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            if child.kind() == "pair" {
                self.push_toml_pair(child, Some(key.as_str()), &mut symbol.children);
            }
        }

        symbols.push(symbol);
    }

    fn push_toml_pair(
        &mut self,
        node: Node<'_>,
        parent_key: Option<&str>,
        symbols: &mut Vec<Symbol>,
    ) {
        let Some(key_node) = first_key_child(node) else {
            return;
        };
        let Some(name) = self.toml_key(key_node) else {
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

        if let Some(value) = value_after_key(node, key_node) {
            self.collect_toml_value(value, Some(key.as_str()), &mut symbol.children);
        }

        symbols.push(symbol);
    }

    fn collect_toml_value(
        &mut self,
        node: Node<'_>,
        parent_key: Option<&str>,
        symbols: &mut Vec<Symbol>,
    ) {
        match node.kind() {
            "inline_table" => {
                let mut cursor = node.walk();
                for child in node.named_children(&mut cursor) {
                    if child.kind() == "pair" {
                        self.push_toml_pair(child, parent_key, symbols);
                    }
                }
            }
            "array" => {
                let mut cursor = node.walk();
                for child in node.named_children(&mut cursor) {
                    self.collect_toml_value(child, parent_key, symbols);
                }
            }
            _ => {}
        }
    }

    fn node_text(&self, node: Node<'_>) -> Option<&'a str> {
        node.utf8_text(self.source.as_bytes()).ok()
    }

    fn toml_key(&self, node: Node<'_>) -> Option<String> {
        match node.kind() {
            "dotted_key" => {
                let mut parts = Vec::new();
                let mut cursor = node.walk();
                for child in node.named_children(&mut cursor) {
                    if is_key_child(child) {
                        parts.push(self.toml_key(child)?);
                    }
                }
                Some(parts.join("."))
            }
            "bare_key" | "quoted_key" => self.node_text(node).map(strip_quotes),
            _ => None,
        }
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

fn first_key_child(node: Node<'_>) -> Option<Node<'_>> {
    let mut cursor = node.walk();
    let key = node
        .named_children(&mut cursor)
        .find(|child| is_key_child(*child));
    key
}

fn value_after_key<'a>(node: Node<'a>, key_node: Node<'a>) -> Option<Node<'a>> {
    let mut seen_key = false;
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        if child == key_node {
            seen_key = true;
            continue;
        }
        if seen_key && !is_key_child(child) {
            return Some(child);
        }
    }
    None
}

fn toml_table_span(node: Node<'_>) -> LineSpan {
    let mut end_line = node.start_position().row + 1;
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        if child.kind() == "pair" {
            end_line = child.end_position().row + 1;
        }
    }
    LineSpan::new(node.start_position().row + 1, end_line)
}

fn is_key_child(node: Node<'_>) -> bool {
    matches!(node.kind(), "bare_key" | "dotted_key" | "quoted_key")
}

fn strip_quotes(text: &str) -> String {
    let text = text.trim();
    text.strip_prefix('"')
        .and_then(|text| text.strip_suffix('"'))
        .or_else(|| {
            text.strip_prefix('\'')
                .and_then(|text| text.strip_suffix('\''))
        })
        .unwrap_or(text)
        .to_owned()
}

fn line_span(node: Node<'_>) -> LineSpan {
    LineSpan::new(node.start_position().row + 1, node.end_position().row + 1)
}
