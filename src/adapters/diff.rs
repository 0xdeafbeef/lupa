use std::collections::HashMap;
use std::path::Path;

use tree_sitter::{Node, Parser, Point};

use crate::grammars;
use crate::model::{FileMap, Language, LineSpan, ParseError, Symbol, SymbolKind};

pub fn parse(path: &Path, source: String) -> FileMap {
    let mut parser = Parser::new();
    let mut parse_errors = Vec::new();
    let Some(language) = grammars::language(Language::Diff) else {
        parse_errors.push(ParseError {
            line: 1,
            message: "failed to load diff grammar".to_owned(),
        });
        return file_map(path, source, Vec::new(), parse_errors);
    };

    if let Err(err) = parser.set_language(&language) {
        parse_errors.push(ParseError {
            line: 1,
            message: format!("failed to load diff grammar: {err}"),
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
    let mut file = FileMap::new(path.to_path_buf(), Language::Diff, source, symbols);
    file.parse_errors = parse_errors;
    file
}

fn collect_parse_errors(node: Node<'_>, parse_errors: &mut Vec<ParseError>) {
    if node.is_error() || node.is_missing() {
        parse_errors.push(ParseError {
            line: node.start_position().row + 1,
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
        let mut cursor = root.walk();
        let children: Vec<_> = root.named_children(&mut cursor).collect();
        let mut symbols = Vec::new();
        let mut index = 0;

        while index < children.len() {
            match children[index].kind() {
                "block" => {
                    self.push_block(children[index], &mut symbols);
                    index += 1;
                }
                "old_file"
                    if children
                        .get(index + 1)
                        .is_some_and(|child| child.kind() == "new_file") =>
                {
                    let end = children[index + 2..]
                        .iter()
                        .position(|child| matches!(child.kind(), "block" | "old_file"))
                        .map_or(children.len(), |offset| index + 2 + offset);
                    self.push_loose_file(&children[index..end], &mut symbols);
                    index = end;
                }
                _ => index += 1,
            }
        }

        symbols
    }

    fn push_block(&mut self, node: Node<'_>, symbols: &mut Vec<Symbol>) {
        let mut cursor = node.walk();
        let children: Vec<_> = node.named_children(&mut cursor).collect();
        let old_file = child_of_kind(&children, "old_file");
        let new_file = child_of_kind(&children, "new_file");
        let command = child_of_kind(&children, "command");
        let Some(path) = self.patch_path(old_file, new_file, command) else {
            return;
        };

        let key = self.unique_key(path.clone());
        let signature_node = command.or(new_file).or(old_file).unwrap_or(node);
        let mut symbol = Symbol::new(
            key.clone(),
            SymbolKind::Heading,
            path,
            self.signature(signature_node),
            boundary_line_span(node),
        );

        if let Some(hunks) = child_of_kind(&children, "hunks") {
            let mut cursor = hunks.walk();
            for hunk in hunks.named_children(&mut cursor) {
                if hunk.kind() == "hunk" {
                    self.push_hunk(hunk, key.as_str(), &mut symbol.children);
                }
            }
        }

        symbols.push(symbol);
    }

    fn push_hunk(&mut self, node: Node<'_>, parent_key: &str, symbols: &mut Vec<Symbol>) {
        let Some(location) = node.child_by_field_name("location") else {
            return;
        };
        let key = self.unique_key(format!("{parent_key}.hunk"));
        let mut symbol = Symbol::new(
            key,
            SymbolKind::Node,
            "hunk",
            self.signature(location),
            boundary_line_span(node),
        );
        symbol.parent_key = Some(parent_key.to_owned());
        symbols.push(symbol);
    }

    fn push_loose_file(&mut self, nodes: &[Node<'_>], symbols: &mut Vec<Symbol>) {
        let Some(old_file) = nodes.first().copied() else {
            return;
        };
        let Some(new_file) = nodes.get(1).copied() else {
            return;
        };
        let Some(path) = self.patch_path(Some(old_file), Some(new_file), None) else {
            return;
        };

        let key = self.unique_key(path.clone());
        let end_node = nodes.last().copied().unwrap_or(new_file);
        let mut symbol = Symbol::new(
            key.clone(),
            SymbolKind::Heading,
            path,
            self.signature(new_file),
            boundary_span(old_file.start_position(), end_node.end_position()),
        );

        let location_indexes: Vec<_> = nodes
            .iter()
            .enumerate()
            .filter_map(|(index, node)| (node.kind() == "location").then_some(index))
            .collect();
        for (location_index, next_location_index) in location_indexes.iter().zip(
            location_indexes
                .iter()
                .skip(1)
                .copied()
                .chain(std::iter::once(nodes.len())),
        ) {
            let location = nodes[*location_index];
            let end = nodes[next_location_index - 1];
            let hunk_key = self.unique_key(format!("{key}.hunk"));
            let mut hunk = Symbol::new(
                hunk_key,
                SymbolKind::Node,
                "hunk",
                self.signature(location),
                boundary_span(location.start_position(), end.end_position()),
            );
            hunk.parent_key = Some(key.clone());
            symbol.children.push(hunk);
        }

        symbols.push(symbol);
    }

    fn patch_path(
        &self,
        old_file: Option<Node<'_>>,
        new_file: Option<Node<'_>>,
        command: Option<Node<'_>>,
    ) -> Option<String> {
        let old_path = old_file.and_then(|node| self.file_path(node));
        let new_path = new_file.and_then(|node| self.file_path(node));
        let selected = new_path
            .filter(|path| path != "/dev/null")
            .or(old_path)
            .or_else(|| self.command_path(command?))?;
        Some(normalize_patch_path(&selected))
    }

    fn file_path(&self, node: Node<'_>) -> Option<String> {
        let mut cursor = node.walk();
        let filename = node
            .named_children(&mut cursor)
            .find(|child| child.kind() == "filename")?;
        self.node_text(filename)
            .and_then(|text| text.split('\t').next())
            .map(str::trim)
            .filter(|text| !text.is_empty())
            .map(str::to_owned)
    }

    fn command_path(&self, node: Node<'_>) -> Option<String> {
        let mut cursor = node.walk();
        let filename = node
            .named_children(&mut cursor)
            .find(|child| child.kind() == "filename")?;
        self.node_text(filename)?
            .split_ascii_whitespace()
            .next_back()
            .map(str::to_owned)
    }

    fn node_text(&self, node: Node<'_>) -> Option<&'a str> {
        node.utf8_text(self.source.as_bytes()).ok()
    }

    fn signature(&self, node: Node<'_>) -> String {
        self.node_text(node)
            .and_then(|text| text.lines().next())
            .map(str::trim)
            .unwrap_or_default()
            .to_owned()
    }

    fn unique_key(&mut self, key: String) -> String {
        let count = self.key_counts.entry(key.clone()).or_insert(0);
        *count += 1;
        if *count == 1 {
            return key;
        }
        format!("{key}#{count}")
    }
}

fn child_of_kind<'a>(nodes: &'a [Node<'a>], kind: &str) -> Option<Node<'a>> {
    nodes.iter().copied().find(|node| node.kind() == kind)
}

fn normalize_patch_path(path: &str) -> String {
    path.strip_prefix("a/")
        .or_else(|| path.strip_prefix("b/"))
        .unwrap_or(path)
        .to_owned()
}

fn boundary_line_span(node: Node<'_>) -> LineSpan {
    boundary_span(node.start_position(), node.end_position())
}

fn boundary_span(start: Point, end: Point) -> LineSpan {
    let start_line = start.row + 1;
    let mut end_line = end.row + 1;
    if end.column == 0 && end_line > start_line {
        end_line -= 1;
    }
    LineSpan::new(start_line, end_line)
}
