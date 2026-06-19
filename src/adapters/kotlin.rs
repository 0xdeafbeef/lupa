use std::collections::BTreeMap;
use std::path::Path;

use tree_sitter::{Node, Parser};

use crate::grammars;
use crate::model::{FileMap, Language, LineSpan, ParseError, Symbol, SymbolKind};

pub fn parse(path: &Path, source: String) -> FileMap {
    let mut parser = Parser::new();
    let mut parse_errors = Vec::new();
    let Some(language) = grammars::language(Language::Kotlin) else {
        parse_errors.push(ParseError {
            line: 1,
            message: "failed to load Kotlin grammar".to_owned(),
        });
        return file_map(path, source, Vec::new(), parse_errors);
    };

    if let Err(err) = parser.set_language(&language) {
        parse_errors.push(ParseError {
            line: 1,
            message: format!("failed to load Kotlin grammar: {err}"),
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
    let mut file = FileMap::new(path.to_path_buf(), Language::Kotlin, source, symbols);
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
    key_counts: BTreeMap<String, usize>,
}

impl<'a> Collector<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            key_counts: BTreeMap::new(),
        }
    }

    fn collect(mut self, root: Node<'_>) -> Vec<Symbol> {
        let mut symbols = Vec::new();
        self.collect_container(root, "", None, &mut symbols);
        symbols
    }

    fn collect_container(
        &mut self,
        node: Node<'_>,
        prefix: &str,
        parent_key: Option<&str>,
        symbols: &mut Vec<Symbol>,
    ) {
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            match child.kind() {
                "class_declaration" | "object_declaration" | "companion_object" => {
                    if let Some(symbol) = self.class_symbol(child, prefix, parent_key) {
                        symbols.push(symbol);
                    }
                }
                "function_declaration" => {
                    if let Some(symbol) = self.function_symbol(child, prefix, parent_key) {
                        symbols.push(symbol);
                    }
                }
                "property_declaration" => {
                    if let Some(symbol) = self.property_symbol(child, prefix, parent_key) {
                        symbols.push(symbol);
                    }
                }
                "call_expression" if parent_key.is_none() => {
                    if let Some(symbol) = self.call_symbol(child) {
                        symbols.push(symbol);
                    }
                }
                _ => {}
            }
        }
    }

    fn class_symbol(
        &mut self,
        node: Node<'_>,
        prefix: &str,
        parent_key: Option<&str>,
    ) -> Option<Symbol> {
        let name = if node.kind() == "companion_object" {
            self.type_identifier(node)
                .unwrap_or_else(|| "Companion".to_owned())
        } else {
            self.type_identifier(node)?
        };
        let key = self.unique_key(&prefixed_key(prefix, &name));
        let mut symbol = self.item_symbol(
            node,
            class_kind(node),
            &name,
            &key,
            parent_key,
            Some("class_body"),
        );
        if let Some(body) = class_body(node) {
            self.collect_container(body, &key, Some(&key), &mut symbol.children);
        }
        Some(symbol)
    }

    fn function_symbol(
        &mut self,
        node: Node<'_>,
        prefix: &str,
        parent_key: Option<&str>,
    ) -> Option<Symbol> {
        let name = self.function_name(node)?;
        let key = self.unique_key(&prefixed_key(prefix, &name));
        Some(self.item_symbol(
            node,
            function_kind(parent_key),
            &name,
            &key,
            parent_key,
            None,
        ))
    }

    fn property_symbol(
        &mut self,
        node: Node<'_>,
        prefix: &str,
        parent_key: Option<&str>,
    ) -> Option<Symbol> {
        let name = self.variable_name(node)?;
        let key = self.unique_key(&prefixed_key(prefix, &name));
        Some(self.item_symbol(node, SymbolKind::Field, &name, &key, parent_key, None))
    }

    fn call_symbol(&mut self, node: Node<'_>) -> Option<Symbol> {
        let name = self.call_name(node)?;
        let key = self.unique_key(&name);
        Some(self.item_symbol(node, SymbolKind::Function, &name, &key, None, None))
    }

    fn item_symbol(
        &self,
        node: Node<'_>,
        kind: SymbolKind,
        name: &str,
        key: &str,
        parent_key: Option<&str>,
        body_kind: Option<&str>,
    ) -> Symbol {
        let mut symbol = Symbol::new(key, kind, name, self.signature(node), line_span(node));
        symbol.body_range = body_kind
            .and_then(|kind| child_of_kind(node, kind))
            .map(line_span);
        symbol.parent_key = parent_key.map(str::to_owned);
        symbol
    }

    fn type_identifier(&self, node: Node<'_>) -> Option<String> {
        first_named_child_of_kinds(node, &["type_identifier", "identifier"])
            .and_then(|node| self.node_text(node))
    }

    fn function_name(&self, node: Node<'_>) -> Option<String> {
        let mut cursor = node.walk();
        let mut receiver = None;
        for child in node.named_children(&mut cursor) {
            match child.kind() {
                "receiver_type" | "user_type" if receiver.is_none() => {
                    receiver = Some(child);
                }
                "simple_identifier" | "identifier" => {
                    let name = self.node_text(child)?;
                    let Some(receiver) = receiver else {
                        return Some(name);
                    };
                    let separator = &self.source[receiver.end_byte()..child.start_byte()];
                    return if separator.contains('.') {
                        let receiver = self.node_text(receiver)?;
                        Some(format!("{receiver}.{name}"))
                    } else {
                        Some(name)
                    };
                }
                _ => {}
            }
        }
        None
    }

    fn variable_name(&self, node: Node<'_>) -> Option<String> {
        child_of_kind(node, "variable_declaration")
            .and_then(|declaration| {
                first_named_child_of_kinds(declaration, &["simple_identifier", "identifier"])
            })
            .and_then(|node| self.node_text(node))
    }

    fn call_name(&self, node: Node<'_>) -> Option<String> {
        let first = node.named_child(0)?;
        matches!(first.kind(), "simple_identifier" | "identifier")
            .then(|| self.node_text(first))
            .flatten()
    }

    fn node_text(&self, node: Node<'_>) -> Option<String> {
        node.utf8_text(self.source.as_bytes())
            .ok()
            .map(str::trim)
            .filter(|text| !text.is_empty())
            .map(str::to_owned)
    }

    fn unique_key(&mut self, base: &str) -> String {
        let count = self.key_counts.entry(base.to_owned()).or_insert(0);
        *count += 1;
        if *count == 1 {
            base.to_owned()
        } else {
            format!("{base}#{count}")
        }
    }

    fn signature(&self, node: Node<'_>) -> String {
        let end_byte = child_of_kind(node, "class_body")
            .or_else(|| child_of_kind(node, "enum_class_body"))
            .or_else(|| child_of_kind(node, "function_body"))
            .map_or_else(|| node.end_byte(), |body| body.start_byte());
        collapse_whitespace(
            self.source[node.start_byte()..end_byte]
                .trim()
                .trim_end_matches('='),
        )
    }
}

fn class_kind(node: Node<'_>) -> SymbolKind {
    if has_child_kind(node, "interface") || has_descendant_kind(node, "interface") {
        SymbolKind::Interface
    } else if has_child_kind(node, "enum") || has_descendant_kind(node, "enum") {
        SymbolKind::Enum
    } else {
        SymbolKind::Class
    }
}

fn function_kind(parent_key: Option<&str>) -> SymbolKind {
    if parent_key.is_some() {
        SymbolKind::Method
    } else {
        SymbolKind::Function
    }
}

fn class_body(node: Node<'_>) -> Option<Node<'_>> {
    child_of_kind(node, "class_body").or_else(|| child_of_kind(node, "enum_class_body"))
}

fn has_child_kind(node: Node<'_>, kind: &str) -> bool {
    let mut cursor = node.walk();
    let found = node.children(&mut cursor).any(|child| child.kind() == kind);
    found
}

fn child_of_kind<'tree>(node: Node<'tree>, kind: &str) -> Option<Node<'tree>> {
    let mut cursor = node.walk();
    let found = node
        .named_children(&mut cursor)
        .find(|child| child.kind() == kind);
    found
}

fn first_named_child_of_kinds<'tree>(node: Node<'tree>, kinds: &[&str]) -> Option<Node<'tree>> {
    let mut cursor = node.walk();
    let found = node
        .named_children(&mut cursor)
        .find(|child| kinds.contains(&child.kind()));
    found
}

fn has_descendant_kind(node: Node<'_>, kind: &str) -> bool {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == kind || has_descendant_kind(child, kind) {
            return true;
        }
    }
    false
}

fn prefixed_key(prefix: &str, name: &str) -> String {
    if prefix.is_empty() {
        name.to_owned()
    } else {
        format!("{prefix}.{name}")
    }
}

fn collapse_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn line_span(node: Node<'_>) -> LineSpan {
    LineSpan::new(node.start_position().row + 1, node.end_position().row + 1)
}
