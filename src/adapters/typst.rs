use std::collections::HashMap;
use std::path::Path;

use tree_sitter::{Node, Parser};

use crate::model::{FileMap, Language, LineSpan, ParseError, Symbol, SymbolKind};

#[derive(Debug)]
struct Heading {
    level: usize,
    start_line: usize,
    end_line: usize,
    name: String,
    signature: String,
}

pub fn parse(path: &Path, source: String) -> FileMap {
    let mut parser = Parser::new();
    let language = arborium_typst::language().into();
    let mut parse_errors = Vec::new();

    if let Err(err) = parser.set_language(&language) {
        parse_errors.push(ParseError {
            line: 1,
            message: format!("failed to load Typst grammar: {err}"),
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
    let mut file = FileMap::new(path.to_path_buf(), Language::Typst, source, symbols);
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
        let mut headings = Vec::new();
        let mut symbols = Vec::new();
        self.collect_node(root, &mut headings, &mut symbols);

        set_end_lines(&mut headings, count_lines(self.source));
        let mut next = 0;
        symbols.extend(build_heading_symbols(
            &headings,
            &mut next,
            0,
            None,
            &mut self.key_counts,
        ));
        symbols.sort_by_key(|symbol| symbol.range.start_line);
        symbols
    }

    fn collect_node(
        &mut self,
        node: Node<'_>,
        headings: &mut Vec<Heading>,
        symbols: &mut Vec<Symbol>,
    ) {
        match node.kind() {
            "heading" => {
                if let Some(heading) = self.heading(node) {
                    headings.push(heading);
                }
                return;
            }
            "let" => {
                if let Some(symbol) = self.let_binding(node) {
                    symbols.push(symbol);
                }
            }
            _ => {}
        }

        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            self.collect_node(child, headings, symbols);
        }
    }

    fn heading(&self, node: Node<'_>) -> Option<Heading> {
        let text = self.node_text(node)?.trim();
        let level = text.bytes().take_while(|byte| *byte == b'=').count();
        if !(1..=6).contains(&level) {
            return None;
        }

        let name = text[level..].trim_matches([' ', '\t']).to_owned();
        Some(Heading {
            level,
            start_line: line_span(node).start_line,
            end_line: line_span(node).start_line,
            name,
            signature: text.to_owned(),
        })
    }

    fn let_binding(&mut self, node: Node<'_>) -> Option<Symbol> {
        if !self.is_markup_let(node) {
            return None;
        }

        let pattern = node.child_by_field_name("pattern")?;
        let name = self.node_text(pattern)?.trim();
        if name.is_empty() {
            return None;
        }

        let key = unique_key(name, &mut self.key_counts);
        Some(Symbol::new(
            key,
            SymbolKind::Field,
            name,
            self.signature(node),
            line_span(node),
        ))
    }

    fn is_markup_let(&self, node: Node<'_>) -> bool {
        let line = self.source_line(node);
        let column = node.start_position().column;
        let prefix = line.get(..column).unwrap_or("");
        prefix.trim_matches([' ', '\t']) == "#" || line.trim_start().starts_with("#let")
    }

    fn signature(&self, node: Node<'_>) -> String {
        self.source_line(node).trim_end().to_owned()
    }

    fn source_line(&self, node: Node<'_>) -> &'a str {
        self.source
            .lines()
            .nth(node.start_position().row)
            .unwrap_or("")
    }

    fn node_text(&self, node: Node<'_>) -> Option<&'a str> {
        node.utf8_text(self.source.as_bytes()).ok()
    }
}

fn set_end_lines(headings: &mut [Heading], line_count: usize) {
    let mut next_by_level: [Option<usize>; 7] = [None; 7];

    for heading in headings.iter_mut().rev() {
        let next_line = next_by_level
            .iter()
            .take(heading.level + 1)
            .skip(1)
            .flatten()
            .copied()
            .min();
        heading.end_line = next_line.map_or(line_count, |line| line.saturating_sub(1));
        next_by_level[heading.level] = Some(heading.start_line);
    }
}

fn build_heading_symbols(
    headings: &[Heading],
    next: &mut usize,
    parent_level: usize,
    parent_key: Option<&str>,
    key_counts: &mut HashMap<String, usize>,
) -> Vec<Symbol> {
    let mut symbols = Vec::new();

    while let Some(heading) = headings.get(*next) {
        if heading.level <= parent_level {
            break;
        }

        let leaf_key = unique_key(&heading.name, key_counts);
        let key =
            parent_key.map_or_else(|| leaf_key.clone(), |parent| format!("{parent}.{leaf_key}"));
        let mut symbol = Symbol::new(
            key.clone(),
            SymbolKind::Heading,
            heading.name.clone(),
            heading.signature.clone(),
            LineSpan::new(heading.start_line, heading.end_line),
        );
        symbol.parent_key = parent_key.map(str::to_owned);

        *next += 1;
        symbol.children = build_heading_symbols(
            headings,
            next,
            heading.level,
            Some(key.as_str()),
            key_counts,
        );
        symbols.push(symbol);
    }

    symbols
}

fn unique_key(name: &str, key_counts: &mut HashMap<String, usize>) -> String {
    let base = if name.is_empty() { "heading" } else { name };
    let count = key_counts.entry(base.to_owned()).or_insert(0);
    *count += 1;

    if *count == 1 {
        base.to_owned()
    } else {
        format!("{base}#{count}")
    }
}

fn line_span(node: Node<'_>) -> LineSpan {
    LineSpan::new(
        node.start_position().row + 1,
        node.end_position().row.saturating_add(1),
    )
}

fn count_lines(source: &str) -> usize {
    if source.is_empty() {
        0
    } else {
        source.lines().count()
    }
}
