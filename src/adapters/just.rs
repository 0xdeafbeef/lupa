use std::collections::HashMap;
use std::path::Path;

use arborium::tree_sitter::{Node, Parser};

use crate::model::{FileMap, Language, LineSpan, ParseError, Symbol, SymbolKind};

pub fn parse(path: &Path, source: String) -> FileMap {
    let mut parser = Parser::new();
    let mut parse_errors = Vec::new();
    let Some(language) = arborium::get_language("just") else {
        parse_errors.push(ParseError {
            line: 1,
            message: "failed to load Just grammar: Arborium grammar 'just' is not enabled"
                .to_owned(),
        });
        return file_map(path, source, Vec::new(), parse_errors);
    };

    if let Err(err) = parser.set_language(&language) {
        parse_errors.push(ParseError {
            line: 1,
            message: format!("failed to load Just grammar: {err}"),
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
    let mut file = FileMap::new(path.to_path_buf(), Language::Just, source, symbols);
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
        let mut cursor = root.walk();

        for child in root.named_children(&mut cursor) {
            match child.kind() {
                "assignment" => self.push_assignment(child, child, &mut symbols),
                "export" => self.push_export(child, &mut symbols),
                "alias" => self.push_alias(child, &mut symbols),
                "recipe" => self.push_recipe(child, &mut symbols),
                _ => {}
            }
        }

        symbols
    }

    fn push_export(&mut self, node: Node<'_>, symbols: &mut Vec<Symbol>) {
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            if child.kind() == "assignment" {
                self.push_assignment(child, node, symbols);
                return;
            }
        }
    }

    fn push_assignment(&mut self, node: Node<'_>, range_node: Node<'_>, symbols: &mut Vec<Symbol>) {
        let Some(name) = self.field_text(node, "left") else {
            return;
        };

        let key = unique_key(name, &mut self.key_counts);
        symbols.push(Symbol::new(
            key,
            SymbolKind::Field,
            name,
            self.signature(range_node),
            line_span(range_node),
        ));
    }

    fn push_alias(&mut self, node: Node<'_>, symbols: &mut Vec<Symbol>) {
        let Some(name) = self.field_text(node, "left") else {
            return;
        };

        let key = unique_key(name, &mut self.key_counts);
        symbols.push(Symbol::new(
            key,
            SymbolKind::Field,
            name,
            self.signature(node),
            line_span(node),
        ));
    }

    fn push_recipe(&mut self, node: Node<'_>, symbols: &mut Vec<Symbol>) {
        let Some(header) = first_named_child(node, "recipe_header") else {
            return;
        };
        let Some(name) = self.field_text(header, "name") else {
            return;
        };

        let key = unique_key(name, &mut self.key_counts);
        symbols.push(Symbol::new(
            key,
            SymbolKind::Function,
            name,
            self.signature(header),
            line_span(node),
        ));
    }

    fn field_text(&self, node: Node<'_>, field: &str) -> Option<&'a str> {
        let child = node.child_by_field_name(field)?;
        self.node_text(child)
            .map(str::trim)
            .filter(|text| !text.is_empty())
    }

    fn signature(&self, node: Node<'_>) -> String {
        self.source
            .lines()
            .nth(node.start_position().row)
            .unwrap_or("")
            .trim_end()
            .to_owned()
    }

    fn node_text(&self, node: Node<'_>) -> Option<&'a str> {
        node.utf8_text(self.source.as_bytes()).ok()
    }
}

fn first_named_child<'a>(node: Node<'a>, kind: &str) -> Option<Node<'a>> {
    let mut cursor = node.walk();
    let child = node
        .named_children(&mut cursor)
        .find(|child| child.kind() == kind);
    child
}

fn unique_key(name: &str, key_counts: &mut HashMap<String, usize>) -> String {
    let count = key_counts.entry(name.to_owned()).or_insert(0);
    *count += 1;

    if *count == 1 {
        name.to_owned()
    } else {
        format!("{name}#{count}")
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
