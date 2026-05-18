use std::collections::BTreeMap;
use std::path::Path;

use tree_sitter::{Node, Parser};

use crate::model::{FileMap, Language, LineSpan, ParseError, Symbol, SymbolKind};

pub fn parse(path: &Path, source: String) -> FileMap {
    let mut parser = Parser::new();
    let language = tree_sitter_go::LANGUAGE.into();
    let mut parse_errors = Vec::new();

    if let Err(err) = parser.set_language(&language) {
        parse_errors.push(ParseError {
            line: 1,
            message: format!("failed to load Go grammar: {err}"),
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
    let mut file = FileMap::new(path.to_path_buf(), Language::Go, source, symbols);
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
    type_indices: BTreeMap<String, usize>,
    pending_methods: BTreeMap<String, Vec<Symbol>>,
}

impl<'a> Collector<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            key_counts: BTreeMap::new(),
            type_indices: BTreeMap::new(),
            pending_methods: BTreeMap::new(),
        }
    }

    fn collect(mut self, root: Node<'_>) -> Vec<Symbol> {
        let mut symbols = Vec::new();
        let mut cursor = root.walk();
        for child in root.named_children(&mut cursor) {
            match child.kind() {
                "function_declaration" => self.push_function_symbol(child, &mut symbols),
                "method_declaration" => self.push_method_symbol(child, &mut symbols),
                "type_declaration" => self.collect_type_declaration(child, &mut symbols),
                _ => {}
            }
        }
        self.flush_pending_methods(&mut symbols);
        symbols
    }

    fn collect_type_declaration(&mut self, node: Node<'_>, symbols: &mut Vec<Symbol>) {
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            match child.kind() {
                "type_alias" | "type_spec" => self.push_type_symbol(child, symbols),
                _ => {}
            }
        }
    }

    fn push_type_symbol(&mut self, node: Node<'_>, symbols: &mut Vec<Symbol>) {
        let Some(name) = self.node_field_text(node, "name") else {
            return;
        };

        let key = self.unique_key(name);
        let kind = Self::type_symbol_kind(node);
        let mut symbol = self.item_symbol(node, kind, name, &key, None);
        symbol.signature = format!("type {}", self.signature(node));
        if let Some(type_node) = node.child_by_field_name("type") {
            symbol.children = self.type_children(type_node, &key);
        }

        if let Some(mut methods) = self.pending_methods.remove(name) {
            symbol.children.append(&mut methods);
        }

        self.type_indices.insert(name.to_owned(), symbols.len());
        symbols.push(symbol);
    }

    fn type_symbol_kind(node: Node<'_>) -> SymbolKind {
        match node.child_by_field_name("type").map(|node| node.kind()) {
            Some("interface_type") => SymbolKind::Interface,
            Some("struct_type") => SymbolKind::Struct,
            _ => SymbolKind::Type,
        }
    }

    fn type_children(&mut self, node: Node<'_>, parent_key: &str) -> Vec<Symbol> {
        match node.kind() {
            "interface_type" => self.interface_children(node, parent_key),
            "struct_type" => self.struct_children(node, parent_key),
            _ => Vec::new(),
        }
    }

    fn struct_children(&mut self, node: Node<'_>, parent_key: &str) -> Vec<Symbol> {
        let Some(body) = named_child_of_kind(node, "field_declaration_list") else {
            return Vec::new();
        };

        let mut children = Vec::new();
        let mut cursor = body.walk();
        for child in body.named_children(&mut cursor) {
            if child.kind() == "field_declaration" {
                self.push_field_symbols(child, parent_key, &mut children);
            }
        }
        children
    }

    fn interface_children(&mut self, node: Node<'_>, parent_key: &str) -> Vec<Symbol> {
        let mut children = Vec::new();
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            match child.kind() {
                "method_elem" => {
                    if let Some(symbol) = self.interface_method_symbol(child, parent_key) {
                        children.push(symbol);
                    }
                }
                "type_elem" => {
                    if let Some(symbol) = self.interface_type_elem_symbol(child, parent_key) {
                        children.push(symbol);
                    }
                }
                _ => {}
            }
        }
        children
    }

    fn push_field_symbols(&mut self, node: Node<'_>, parent_key: &str, symbols: &mut Vec<Symbol>) {
        let mut cursor = node.walk();
        let names = node
            .children_by_field_name("name", &mut cursor)
            .filter_map(|name| name.utf8_text(self.source.as_bytes()).ok())
            .collect::<Vec<_>>();

        if names.is_empty() {
            if let Some(name) = node
                .child_by_field_name("type")
                .and_then(|type_node| type_name(type_node, self.source.as_bytes()))
            {
                symbols.push(self.field_symbol(node, parent_key, &name));
            }
            return;
        }

        for name in names {
            symbols.push(self.field_symbol(node, parent_key, name));
        }
    }

    fn field_symbol(&mut self, node: Node<'_>, parent_key: &str, name: &str) -> Symbol {
        let key = self.unique_key(&format!("{parent_key}.{name}"));
        let mut symbol = Symbol::new(
            key,
            SymbolKind::Field,
            name,
            self.collapsed_text(node),
            line_span(node),
        );
        symbol.parent_key = Some(parent_key.to_owned());
        symbol
    }

    fn interface_method_symbol(&mut self, node: Node<'_>, parent_key: &str) -> Option<Symbol> {
        let name = self.node_field_text(node, "name")?;
        let key = self.unique_key(&format!("{parent_key}.{name}"));
        Some(self.item_symbol(node, SymbolKind::Method, name, &key, Some(parent_key)))
    }

    fn interface_type_elem_symbol(&mut self, node: Node<'_>, parent_key: &str) -> Option<Symbol> {
        let name = self.interface_type_elem_name(node)?;
        let key = self.unique_key(&format!("{parent_key}.{name}"));
        let mut symbol = Symbol::new(
            key,
            SymbolKind::Type,
            &name,
            self.collapsed_text(node),
            line_span(node),
        );
        symbol.parent_key = Some(parent_key.to_owned());
        Some(symbol)
    }

    fn interface_type_elem_name(&self, node: Node<'_>) -> Option<String> {
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            if let Some(name) = type_name(child, self.source.as_bytes()) {
                return Some(name);
            }
        }
        let name = sanitize_key(&self.collapsed_text(node));
        (!name.is_empty()).then_some(name)
    }

    fn push_function_symbol(&mut self, node: Node<'_>, symbols: &mut Vec<Symbol>) {
        let Some(name) = self.node_field_text(node, "name") else {
            return;
        };

        let key = self.unique_key(name);
        symbols.push(self.item_symbol(node, SymbolKind::Function, name, &key, None));
    }

    fn push_method_symbol(&mut self, node: Node<'_>, symbols: &mut [Symbol]) {
        let Some(name) = self.node_field_text(node, "name") else {
            return;
        };
        let Some(receiver) = self.receiver_type(node) else {
            return;
        };

        let key = self.unique_key(&format!("{receiver}.{name}"));
        let symbol = self.item_symbol(node, SymbolKind::Method, name, &key, Some(&receiver));
        self.attach_method(symbols, &receiver, symbol);
    }

    fn attach_method(&mut self, symbols: &mut [Symbol], type_key: &str, method: Symbol) {
        if let Some(idx) = self.type_indices.get(type_key).copied() {
            symbols[idx].children.push(method);
        } else {
            self.pending_methods
                .entry(type_key.to_owned())
                .or_default()
                .push(method);
        }
    }

    fn flush_pending_methods(&mut self, symbols: &mut Vec<Symbol>) {
        for methods in std::mem::take(&mut self.pending_methods).into_values() {
            symbols.extend(methods);
        }
    }

    fn receiver_type(&self, node: Node<'_>) -> Option<String> {
        let receiver = node.child_by_field_name("receiver")?;
        let mut cursor = receiver.walk();
        for child in receiver.named_children(&mut cursor) {
            let type_node = child.child_by_field_name("type").unwrap_or(child);
            if let Some(name) = type_name(type_node, self.source.as_bytes()) {
                return Some(name);
            }
        }
        None
    }

    fn item_symbol(
        &self,
        node: Node<'_>,
        kind: SymbolKind,
        name: &str,
        key: &str,
        parent_key: Option<&str>,
    ) -> Symbol {
        let mut symbol = Symbol::new(key, kind, name, self.signature(node), line_span(node));
        symbol.body_range = node.child_by_field_name("body").map(line_span);
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

    fn signature(&self, node: Node<'_>) -> String {
        let end_byte = self.signature_end_byte(node);
        collapse_whitespace(self.source[node.start_byte()..end_byte].trim())
    }

    fn signature_end_byte(&self, node: Node<'_>) -> usize {
        if let Some(body) = node.child_by_field_name("body") {
            return body.start_byte();
        }

        if matches!(node.kind(), "type_alias" | "type_spec") {
            match node.child_by_field_name("type") {
                Some(type_node) if matches!(type_node.kind(), "interface_type" | "struct_type") => {
                    let type_text = self.node_text(type_node);
                    if let Some(open_brace) = type_text.find('{') {
                        return type_node.start_byte() + open_brace;
                    }
                }
                _ => {}
            }
        }

        node.end_byte()
    }

    fn collapsed_text(&self, node: Node<'_>) -> String {
        collapse_whitespace(self.node_text(node).trim())
    }

    fn node_field_text(&self, node: Node<'_>, field_name: &str) -> Option<&'a str> {
        node_field_text(node, field_name, self.source.as_bytes())
    }

    fn node_text(&self, node: Node<'_>) -> &'a str {
        node.utf8_text(self.source.as_bytes()).unwrap_or("")
    }
}

fn node_field_text<'a>(node: Node<'_>, field_name: &str, source: &'a [u8]) -> Option<&'a str> {
    node.child_by_field_name(field_name)
        .and_then(|field| field.utf8_text(source).ok())
}

fn named_child_of_kind<'tree>(node: Node<'tree>, kind: &str) -> Option<Node<'tree>> {
    let mut cursor = node.walk();
    let child = node
        .named_children(&mut cursor)
        .find(|child| child.kind() == kind);
    child
}

fn type_name(node: Node<'_>, source: &[u8]) -> Option<String> {
    match node.kind() {
        "field_identifier" | "identifier" | "type_identifier" => {
            node.utf8_text(source).ok().map(str::to_owned)
        }
        "generic_type" | "negated_type" => node
            .child_by_field_name("type")
            .and_then(|child| type_name(child, source)),
        "qualified_type" => node
            .child_by_field_name("name")
            .and_then(|child| type_name(child, source)),
        _ => {
            let mut cursor = node.walk();
            for child in node.named_children(&mut cursor) {
                if let Some(name) = type_name(child, source) {
                    return Some(name);
                }
            }
            None
        }
    }
}

fn sanitize_key(key: &str) -> String {
    key.chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.') {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

fn collapse_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn line_span(node: Node<'_>) -> LineSpan {
    LineSpan::new(node.start_position().row + 1, node.end_position().row + 1)
}
