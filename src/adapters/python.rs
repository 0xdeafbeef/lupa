use std::collections::BTreeMap;
use std::path::Path;

use tree_sitter::{Node, Parser};

use crate::model::{FileMap, Language, LineSpan, ParseError, Symbol, SymbolKind};

pub fn parse(path: &Path, source: String) -> FileMap {
    let mut parser = Parser::new();
    let language = tree_sitter_python::LANGUAGE.into();
    let mut parse_errors = Vec::new();

    if let Err(err) = parser.set_language(&language) {
        parse_errors.push(ParseError {
            line: 1,
            message: format!("failed to load Python grammar: {err}"),
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
    let mut file = FileMap::new(path.to_path_buf(), Language::Python, source, symbols);
    file.parse_errors = parse_errors;
    file
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
        self.collect_container(root, "", &mut symbols);
        symbols
    }

    fn collect_container(&mut self, node: Node<'_>, prefix: &str, symbols: &mut Vec<Symbol>) {
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            let item = definition_node(child);
            match item.kind() {
                "class_definition" => self.push_class(item, child, prefix, symbols),
                "function_definition" => {
                    if let Some(symbol) =
                        self.function_symbol(item, child, prefix, None, SymbolKind::Function)
                    {
                        symbols.push(symbol);
                    }
                }
                _ => {}
            }
        }
    }

    fn push_class(
        &mut self,
        node: Node<'_>,
        range_node: Node<'_>,
        prefix: &str,
        symbols: &mut Vec<Symbol>,
    ) {
        let Some(name) = self.node_field_text(node, "name") else {
            return;
        };

        let key = self.unique_key(&prefixed_key(prefix, name));
        let mut symbol = self.item_symbol(range_node, node, SymbolKind::Class, name, &key, None);
        if let Some(body) = node.child_by_field_name("body") {
            symbol.children = self.class_methods(body, &key);
        }
        symbols.push(symbol);
    }

    fn class_methods(&mut self, body: Node<'_>, parent_key: &str) -> Vec<Symbol> {
        let mut symbols = Vec::new();
        let mut cursor = body.walk();
        for child in body.named_children(&mut cursor) {
            let item = definition_node(child);
            if item.kind() == "function_definition" {
                if let Some(symbol) = self.function_symbol(
                    item,
                    child,
                    parent_key,
                    Some(parent_key),
                    SymbolKind::Method,
                ) {
                    symbols.push(symbol);
                }
            }
        }
        symbols
    }

    fn function_symbol(
        &mut self,
        node: Node<'_>,
        range_node: Node<'_>,
        prefix: &str,
        parent_key: Option<&str>,
        kind: SymbolKind,
    ) -> Option<Symbol> {
        let name = self.node_field_text(node, "name")?;
        let key = self.unique_key(&prefixed_key(prefix, name));
        Some(self.item_symbol(range_node, node, kind, name, &key, parent_key))
    }

    fn item_symbol(
        &self,
        range_node: Node<'_>,
        declaration_node: Node<'_>,
        kind: SymbolKind,
        name: &str,
        key: &str,
        parent_key: Option<&str>,
    ) -> Symbol {
        let mut symbol = Symbol::new(
            key,
            kind,
            name,
            self.signature(range_node, declaration_node),
            line_span(range_node),
        );
        symbol.body_range = declaration_node.child_by_field_name("body").map(line_span);
        symbol.parent_key = parent_key.map(str::to_owned);
        symbol
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

    fn signature(&self, range_node: Node<'_>, declaration_node: Node<'_>) -> String {
        let end_byte = declaration_node
            .child_by_field_name("body")
            .map_or_else(|| declaration_node.end_byte(), |body| body.start_byte());
        let start_byte = range_node.start_byte();
        collapse_whitespace(
            self.source[start_byte..end_byte]
                .trim()
                .trim_end_matches(':')
                .trim(),
        )
    }

    fn node_field_text(&self, node: Node<'_>, field_name: &str) -> Option<&'a str> {
        node.child_by_field_name(field_name)
            .and_then(|field| field.utf8_text(self.source.as_bytes()).ok())
    }
}

fn definition_node(node: Node<'_>) -> Node<'_> {
    node.child_by_field_name("definition").unwrap_or(node)
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
