use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use tree_sitter::{Node, Parser};

use crate::model::{FileMap, Language, LineSpan, ParseError, Symbol, SymbolKind};

pub fn parse(path: &Path, source: String) -> FileMap {
    let mut parser = Parser::new();
    let language = tree_sitter_rust::LANGUAGE.into();
    let mut parse_errors = Vec::new();

    if let Err(err) = parser.set_language(&language) {
        parse_errors.push(ParseError {
            line: 1,
            message: format!("failed to load Rust grammar: {err}"),
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

    let mut local_types = BTreeSet::new();
    collect_local_types(root, "", source.as_bytes(), &mut local_types);

    let symbols = Collector::new(&source, local_types).collect(root);
    file_map(path, source, symbols, parse_errors)
}

fn file_map(
    path: &Path,
    source: String,
    symbols: Vec<Symbol>,
    parse_errors: Vec<ParseError>,
) -> FileMap {
    let mut file = FileMap::new(path.to_path_buf(), Language::Rust, source, symbols);
    file.parse_errors = parse_errors;
    file
}

fn collect_local_types(
    node: Node<'_>,
    prefix: &str,
    source: &[u8],
    local_types: &mut BTreeSet<String>,
) {
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        match child.kind() {
            "enum_item" | "struct_item" | "trait_item" => {
                if let Some(name) = node_field_text(child, "name", source) {
                    local_types.insert(prefixed_key(prefix, name));
                }
            }
            "mod_item" => {
                if let (Some(name), Some(body)) = (
                    node_field_text(child, "name", source),
                    child.child_by_field_name("body"),
                ) {
                    let prefix = prefixed_key(prefix, name);
                    collect_local_types(body, &prefix, source, local_types);
                }
            }
            _ => {}
        }
    }
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
    local_types: BTreeSet<String>,
    key_counts: BTreeMap<String, usize>,
    type_indices: BTreeMap<String, usize>,
    pending_methods: BTreeMap<String, Vec<Symbol>>,
}

#[derive(Default)]
struct Attributes {
    values: Vec<String>,
    start_line: Option<usize>,
}

impl Attributes {
    fn clear(&mut self) {
        self.values.clear();
        self.start_line = None;
    }

    fn take(&mut self) -> Self {
        Self {
            values: std::mem::take(&mut self.values),
            start_line: self.start_line.take(),
        }
    }
}

impl<'a> Collector<'a> {
    fn new(source: &'a str, local_types: BTreeSet<String>) -> Self {
        Self {
            source,
            local_types,
            key_counts: BTreeMap::new(),
            type_indices: BTreeMap::new(),
            pending_methods: BTreeMap::new(),
        }
    }

    fn collect(mut self, root: Node<'_>) -> Vec<Symbol> {
        let mut symbols = Vec::new();
        self.collect_container(root, "", &mut symbols);
        symbols
    }

    fn collect_container(&mut self, node: Node<'_>, prefix: &str, symbols: &mut Vec<Symbol>) {
        let mut attrs = Attributes::default();
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            match child.kind() {
                "attribute_item" => self.push_attribute(child, &mut attrs),
                "enum_item" => self.push_type_symbol(
                    child,
                    prefix,
                    SymbolKind::Enum,
                    Self::enum_children,
                    symbols,
                    attrs.take(),
                ),
                "function_item" => {
                    if let Some(symbol) = self.function_symbol(
                        child,
                        prefix,
                        None,
                        SymbolKind::Function,
                        attrs.take(),
                    ) {
                        symbols.push(symbol);
                    }
                }
                "impl_item" => self.push_impl_symbol(child, prefix, symbols, attrs.take()),
                "mod_item" => {
                    attrs.clear();
                    self.collect_module(child, prefix, symbols);
                }
                "struct_item" => self.push_type_symbol(
                    child,
                    prefix,
                    SymbolKind::Struct,
                    Self::struct_children,
                    symbols,
                    attrs.take(),
                ),
                "trait_item" => self.push_type_symbol(
                    child,
                    prefix,
                    SymbolKind::Trait,
                    Self::trait_children,
                    symbols,
                    attrs.take(),
                ),
                kind if is_comment_kind(kind) => {}
                _ => attrs.clear(),
            }
        }
    }

    fn collect_module(&mut self, node: Node<'_>, prefix: &str, symbols: &mut Vec<Symbol>) {
        let Some(name) = self.node_field_text(node, "name") else {
            return;
        };
        let Some(body) = node.child_by_field_name("body") else {
            return;
        };
        let prefix = prefixed_key(prefix, name);
        self.collect_container(body, &prefix, symbols);
    }

    fn push_type_symbol(
        &mut self,
        node: Node<'_>,
        prefix: &str,
        kind: SymbolKind,
        children: fn(&mut Self, Node<'_>, &str) -> Vec<Symbol>,
        symbols: &mut Vec<Symbol>,
        attrs: Attributes,
    ) {
        let Some(name) = self.node_field_text(node, "name") else {
            return;
        };

        let base_key = prefixed_key(prefix, name);
        let key = self.unique_key(&base_key);
        let mut symbol = self.item_symbol(node, kind, name, &key, None, attrs);
        symbol.children = children(self, node, &key);

        if let Some(mut methods) = self.pending_methods.remove(&base_key) {
            symbol.children.append(&mut methods);
        }

        self.type_indices.insert(base_key, symbols.len());
        symbols.push(symbol);
    }

    fn struct_children(&mut self, node: Node<'_>, parent_key: &str) -> Vec<Symbol> {
        let Some(body) = node.child_by_field_name("body") else {
            return Vec::new();
        };

        match body.kind() {
            "field_declaration_list" => self.named_field_symbols(body, parent_key),
            "ordered_field_declaration_list" => self.ordered_field_symbols(body, parent_key),
            _ => Vec::new(),
        }
    }

    fn enum_children(&mut self, node: Node<'_>, parent_key: &str) -> Vec<Symbol> {
        let Some(body) = node.child_by_field_name("body") else {
            return Vec::new();
        };

        let mut children = Vec::new();
        let mut attrs = Attributes::default();
        let mut cursor = body.walk();
        for child in body.named_children(&mut cursor) {
            match child.kind() {
                "attribute_item" => {
                    self.push_attribute(child, &mut attrs);
                    continue;
                }
                kind if is_comment_kind(kind) => continue,
                "enum_variant" => {}
                _ => {
                    attrs.clear();
                    continue;
                }
            }

            {
                let Some(name) = self.node_field_text(child, "name") else {
                    attrs.clear();
                    continue;
                };
                let key = self.unique_key(&format!("{parent_key}.{name}"));
                let mut symbol = self.item_symbol(
                    child,
                    SymbolKind::Field,
                    name,
                    &key,
                    Some(parent_key),
                    attrs.take(),
                );
                symbol.body_range = child.child_by_field_name("body").map(line_span);
                children.push(symbol);
            }
        }
        children
    }

    fn trait_children(&mut self, node: Node<'_>, parent_key: &str) -> Vec<Symbol> {
        let Some(body) = node.child_by_field_name("body") else {
            return Vec::new();
        };

        let mut children = Vec::new();
        let mut attrs = Attributes::default();
        let mut cursor = body.walk();
        for child in body.named_children(&mut cursor) {
            match child.kind() {
                "attribute_item" => self.push_attribute(child, &mut attrs),
                "function_item" | "function_signature_item" => {
                    if let Some(symbol) =
                        self.method_symbol(child, parent_key, Some(parent_key), attrs.take())
                    {
                        children.push(symbol);
                    }
                }
                kind if is_comment_kind(kind) => {}
                _ => attrs.clear(),
            }
        }
        children
    }

    fn named_field_symbols(&mut self, node: Node<'_>, parent_key: &str) -> Vec<Symbol> {
        let mut children = Vec::new();
        let mut attrs = Attributes::default();
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            match child.kind() {
                "attribute_item" => {
                    self.push_attribute(child, &mut attrs);
                    continue;
                }
                kind if is_comment_kind(kind) => continue,
                "field_declaration" => {}
                _ => {
                    attrs.clear();
                    continue;
                }
            }

            let Some(name) = self.node_field_text(child, "name") else {
                attrs.clear();
                continue;
            };
            let key = self.unique_key(&format!("{parent_key}.{name}"));
            children.push(self.item_symbol(
                child,
                SymbolKind::Field,
                name,
                &key,
                Some(parent_key),
                attrs.take(),
            ));
        }
        children
    }

    fn ordered_field_symbols(&mut self, node: Node<'_>, parent_key: &str) -> Vec<Symbol> {
        let mut children = Vec::new();
        let mut attrs = Attributes::default();
        let mut visibility = None;
        let mut visibility_start_line = None;
        let mut idx = 0;
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            match child.kind() {
                "attribute_item" => {
                    self.push_attribute(child, &mut attrs);
                    continue;
                }
                "visibility_modifier" => {
                    visibility = Some(self.collapsed_text(child));
                    visibility_start_line = Some(child.start_position().row + 1);
                    continue;
                }
                kind if is_comment_kind(kind) => continue,
                _ => {}
            }

            let name = idx.to_string();
            let key = self.unique_key(&format!("{parent_key}.{name}"));
            let type_text = self.collapsed_text(child);
            let signature = if let Some(visibility) = &visibility {
                format!("{name}: {visibility} {type_text}")
            } else {
                format!("{name}: {type_text}")
            };
            let attrs = attrs.take();
            let start_line = attrs.start_line.or(visibility_start_line.take());
            let mut symbol = Symbol::new(
                key,
                SymbolKind::Field,
                name,
                signature,
                line_span_with_attrs(child, start_line),
            );
            symbol.attributes = attrs.values;
            symbol.visibility = visibility.take();
            symbol.parent_key = Some(parent_key.to_owned());
            children.push(symbol);
            idx += 1;
        }
        children
    }

    fn push_impl_symbol(
        &mut self,
        node: Node<'_>,
        prefix: &str,
        symbols: &mut Vec<Symbol>,
        attrs: Attributes,
    ) {
        let Some(type_node) = node.child_by_field_name("type") else {
            return;
        };

        let type_name = type_name(type_node, self.source.as_bytes())
            .unwrap_or_else(|| self.collapsed_text(type_node));
        let trait_name = node
            .child_by_field_name("trait")
            .map(|trait_node| self.collapsed_text(trait_node));
        let impl_key = self.impl_key(&type_name, trait_name.as_deref());
        let mut symbol = self.item_symbol(node, SymbolKind::Impl, "impl", &impl_key, None, attrs);

        let local_type_key = self.local_type_key(prefix, type_node);
        let mut methods = Vec::new();
        if let Some(body) = node.child_by_field_name("body") {
            let method_parent = local_type_key.as_deref().unwrap_or(&symbol.key);
            methods = self.impl_methods(body, method_parent);
        }

        if let Some(type_key) = local_type_key {
            for method in methods {
                self.attach_method(symbols, &type_key, method);
            }
        } else {
            symbol.children = methods;
        }

        symbols.push(symbol);
    }

    fn impl_methods(&mut self, body: Node<'_>, parent_key: &str) -> Vec<Symbol> {
        let mut methods = Vec::new();
        let mut attrs = Attributes::default();
        let mut cursor = body.walk();
        for child in body.named_children(&mut cursor) {
            match child.kind() {
                "attribute_item" => self.push_attribute(child, &mut attrs),
                "function_item" => {
                    if let Some(symbol) =
                        self.method_symbol(child, parent_key, Some(parent_key), attrs.take())
                    {
                        methods.push(symbol);
                    }
                }
                kind if is_comment_kind(kind) => {}
                _ => attrs.clear(),
            }
        }
        methods
    }

    fn push_attribute(&self, node: Node<'_>, attrs: &mut Attributes) {
        if attrs.start_line.is_none() {
            attrs.start_line = Some(node.start_position().row + 1);
        }
        attrs.values.push(self.collapsed_text(node));
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

    fn function_symbol(
        &mut self,
        node: Node<'_>,
        prefix: &str,
        parent_key: Option<&str>,
        kind: SymbolKind,
        attrs: Attributes,
    ) -> Option<Symbol> {
        let name = self.node_field_text(node, "name")?;
        let base_key = prefixed_key(prefix, name);
        let key = self.unique_key(&base_key);
        Some(self.item_symbol(node, kind, name, &key, parent_key, attrs))
    }

    fn method_symbol(
        &mut self,
        node: Node<'_>,
        parent_key: &str,
        parent: Option<&str>,
        attrs: Attributes,
    ) -> Option<Symbol> {
        let name = self.node_field_text(node, "name")?;
        let key = self.unique_key(&format!("{parent_key}.{name}"));
        Some(self.item_symbol(node, SymbolKind::Method, name, &key, parent, attrs))
    }

    fn item_symbol(
        &self,
        node: Node<'_>,
        kind: SymbolKind,
        name: &str,
        key: &str,
        parent_key: Option<&str>,
        attrs: Attributes,
    ) -> Symbol {
        let mut symbol = Symbol::new(
            key,
            kind,
            name,
            self.signature(node),
            line_span_with_attrs(node, attrs.start_line),
        );
        symbol.attributes = attrs.values;
        symbol.visibility = self.visibility(node);
        symbol.body_range = node.child_by_field_name("body").map(line_span);
        symbol.parent_key = parent_key.map(str::to_owned);
        symbol
    }

    fn impl_key(&mut self, type_name: &str, trait_name: Option<&str>) -> String {
        let base = if let Some(trait_name) = trait_name {
            format!("impl_{trait_name}_for_{type_name}")
        } else {
            format!("impl_{type_name}")
        };
        self.unique_key(&sanitize_key(&base))
    }

    fn local_type_key(&self, prefix: &str, type_node: Node<'_>) -> Option<String> {
        let name = type_name(type_node, self.source.as_bytes())?;
        let scoped = prefixed_key(prefix, &name);
        if self.local_types.contains(&scoped) {
            Some(scoped)
        } else if self.local_types.contains(&name) {
            Some(name)
        } else {
            None
        }
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

    fn visibility(&self, node: Node<'_>) -> Option<String> {
        named_child_of_kind(node, "visibility_modifier")
            .map(|visibility| self.collapsed_text(visibility))
    }

    fn signature(&self, node: Node<'_>) -> String {
        let end_byte = node
            .child_by_field_name("body")
            .map_or_else(|| node.end_byte(), |body| body.start_byte());
        let text = &self.source[node.start_byte()..end_byte];
        collapse_whitespace(text.trim().trim_end_matches(';'))
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

fn is_comment_kind(kind: &str) -> bool {
    matches!(kind, "line_comment" | "block_comment")
}

fn type_name(node: Node<'_>, source: &[u8]) -> Option<String> {
    match node.kind() {
        "identifier" | "type_identifier" => node.utf8_text(source).ok().map(str::to_owned),
        "generic_type" | "qualified_type" | "reference_type" => node
            .child_by_field_name("type")
            .and_then(|child| type_name(child, source)),
        "scoped_type_identifier" => node
            .child_by_field_name("name")
            .and_then(|child| type_name(child, source)),
        _ => {
            let mut found = None;
            let mut cursor = node.walk();
            for child in node.named_children(&mut cursor) {
                if let Some(name) = type_name(child, source) {
                    found = Some(name);
                }
            }
            found
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

fn line_span_with_attrs(node: Node<'_>, start_line: Option<usize>) -> LineSpan {
    let span = line_span(node);
    LineSpan::new(start_line.unwrap_or(span.start_line), span.end_line)
}
