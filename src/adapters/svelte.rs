use std::collections::BTreeMap;
use std::path::Path;

use tree_sitter::{Node, Parser};

use crate::grammars;
use crate::model::{FileMap, Language, LineSpan, ParseError, Symbol, SymbolKind};

pub fn parse(path: &Path, source: String) -> FileMap {
    let mut parser = Parser::new();
    let mut parse_errors = Vec::new();
    let Some(language) = grammars::language(Language::Svelte) else {
        parse_errors.push(ParseError {
            line: 1,
            message: "failed to load Svelte grammar".to_owned(),
        });
        return file_map(path, source, Vec::new(), parse_errors);
    };

    if let Err(err) = parser.set_language(&language) {
        parse_errors.push(ParseError {
            line: 1,
            message: format!("failed to load Svelte grammar: {err}"),
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
    collect_parse_errors(&source, root, &mut parse_errors);
    let symbols = Collector::new(&source).collect(root);
    file_map(path, source, symbols, parse_errors)
}

fn file_map(
    path: &Path,
    source: String,
    symbols: Vec<Symbol>,
    parse_errors: Vec<ParseError>,
) -> FileMap {
    let mut file = FileMap::new(path.to_path_buf(), Language::Svelte, source, symbols);
    file.parse_errors = parse_errors;
    file
}

fn collect_parse_errors(source: &str, node: Node<'_>, parse_errors: &mut Vec<ParseError>) {
    if (node.is_error() || node.is_missing()) && !is_raw_text_ampersand_error(source, node) {
        parse_errors.push(ParseError {
            line: line_span(node).start_line,
            message: format!("parse error in {}", node.kind()),
        });
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        if child.has_error() || child.is_missing() {
            collect_parse_errors(source, child, parse_errors);
        }
    }
}

fn is_raw_text_ampersand_error(source: &str, node: Node<'_>) -> bool {
    if !node.is_error() || node.start_position().row != node.end_position().row {
        return false;
    }

    let Ok(text) = node.utf8_text(source.as_bytes()) else {
        return false;
    };
    let text = text.trim_start();
    if !text.starts_with('&') || text.chars().any(|ch| matches!(ch, '<' | '>' | '{' | '}')) {
        return false;
    }

    // tree-sitter-svelte-next reports raw text like `Edit & fork` as ERROR,
    // but it does not break the surrounding element structure that lupa maps.
    let Some(parent) = node.parent() else {
        return false;
    };
    if parent.kind() == "element" {
        return true;
    }

    parent.kind() == "ERROR"
        && parent
            .parent()
            .is_some_and(|parent| parent.kind() == "element")
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
                "script_element" => {
                    symbols.push(self.named_symbol(
                        child,
                        SymbolKind::Node,
                        "script",
                        prefix,
                        parent_key,
                    ));
                }
                "style_element" => {
                    symbols.push(self.named_symbol(
                        child,
                        SymbolKind::Node,
                        "style",
                        prefix,
                        parent_key,
                    ));
                }
                "element" => {
                    if let Some(symbol) = self.element_symbol(child, prefix, parent_key) {
                        symbols.push(symbol);
                    }
                }
                "if_statement" | "each_statement" | "await_statement" | "key_statement"
                | "snippet_statement" => {
                    let name = self.block_name(child);
                    let mut symbol =
                        self.named_symbol(child, SymbolKind::Node, &name, prefix, parent_key);
                    self.collect_container(
                        child,
                        &symbol.key,
                        Some(&symbol.key),
                        &mut symbol.children,
                    );
                    symbols.push(symbol);
                }
                "render_tag" | "const_tag" => {
                    let name = match child.kind() {
                        "render_tag" => "render",
                        "const_tag" => "const",
                        _ => child.kind(),
                    };
                    symbols.push(self.named_symbol(
                        child,
                        SymbolKind::Node,
                        name,
                        prefix,
                        parent_key,
                    ));
                }
                _ => {
                    self.collect_container(child, prefix, parent_key, symbols);
                }
            }
        }
    }

    fn element_symbol(
        &mut self,
        node: Node<'_>,
        prefix: &str,
        parent_key: Option<&str>,
    ) -> Option<Symbol> {
        let name = self.element_name(node)?;
        let mut symbol = self.named_symbol(node, SymbolKind::Node, &name, prefix, parent_key);
        self.collect_container(node, &symbol.key, Some(&symbol.key), &mut symbol.children);
        Some(symbol)
    }

    fn named_symbol(
        &mut self,
        node: Node<'_>,
        kind: SymbolKind,
        name: &str,
        prefix: &str,
        parent_key: Option<&str>,
    ) -> Symbol {
        let key = self.unique_key(&prefixed_key(prefix, name));
        let mut symbol = Symbol::new(&key, kind, name, self.signature(node), line_span(node));
        symbol.parent_key = parent_key.map(str::to_owned);
        symbol
    }

    fn element_name(&self, node: Node<'_>) -> Option<String> {
        child_of_kind(node, "start_tag")
            .or_else(|| child_of_kind(node, "self_closing_tag"))
            .and_then(|tag| child_of_kind(tag, "tag_name"))
            .and_then(|name| self.node_text(name))
    }

    fn block_name(&self, node: Node<'_>) -> String {
        match node.kind() {
            "if_statement" => "if".to_owned(),
            "each_statement" => "each".to_owned(),
            "await_statement" => "await".to_owned(),
            "key_statement" => "key".to_owned(),
            "snippet_statement" => self
                .descendant_text(node, "snippet_name")
                .unwrap_or_else(|| "snippet".to_owned()),
            _ => node.kind().to_owned(),
        }
    }

    fn descendant_text(&self, node: Node<'_>, kind: &str) -> Option<String> {
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            if child.kind() == kind {
                return self.node_text(child);
            }
            if let Some(text) = self.descendant_text(child, kind) {
                return Some(text);
            }
        }
        None
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
        collapse_whitespace(
            node.utf8_text(self.source.as_bytes())
                .ok()
                .and_then(|text| text.lines().next())
                .unwrap_or_else(|| node.kind())
                .trim(),
        )
    }
}

fn child_of_kind<'tree>(node: Node<'tree>, kind: &str) -> Option<Node<'tree>> {
    let mut cursor = node.walk();
    let found = node
        .named_children(&mut cursor)
        .find(|child| child.kind() == kind);
    found
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
