use std::collections::BTreeMap;
use std::path::Path;

use tree_sitter::{Language as TsLanguage, Node, Parser};

use crate::model::{FileMap, Language, LineSpan, ParseError, Symbol, SymbolKind};

#[derive(Clone, Copy)]
enum LanguageVariant {
    JavaScript,
    TypeScript,
    Tsx,
}

impl LanguageVariant {
    fn from_language(language: Language) -> Self {
        match language {
            Language::JavaScript | Language::Jsx => Self::JavaScript,
            Language::Tsx => Self::Tsx,
            Language::TypeScript => Self::TypeScript,
            Language::Go | Language::Markdown | Language::Python | Language::Rust => {
                unreachable!("javascript adapter only handles JavaScript and TypeScript languages")
            }
        }
    }

    fn language(self) -> TsLanguage {
        match self {
            Self::JavaScript => tree_sitter_javascript::LANGUAGE.into(),
            Self::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            Self::Tsx => tree_sitter_typescript::LANGUAGE_TSX.into(),
        }
    }

    fn grammar_name(self) -> &'static str {
        match self {
            Self::JavaScript => "JavaScript",
            Self::TypeScript => "TypeScript",
            Self::Tsx => "TSX",
        }
    }
}

pub fn parse(path: &Path, language: Language, source: String) -> FileMap {
    let variant = LanguageVariant::from_language(language);
    let mut parser = Parser::new();
    let parser_language = variant.language();
    let mut parse_errors = Vec::new();

    if let Err(err) = parser.set_language(&parser_language) {
        parse_errors.push(ParseError {
            line: 1,
            message: format!("failed to load {} grammar: {err}", variant.grammar_name()),
        });
        return file_map(path, language, source, Vec::new(), parse_errors);
    }

    let Some(tree) = parser.parse(&source, None) else {
        parse_errors.push(ParseError {
            line: 1,
            message: "tree-sitter returned no parse tree".to_owned(),
        });
        return file_map(path, language, source, Vec::new(), parse_errors);
    };

    let root = tree.root_node();
    collect_parse_errors(root, &mut parse_errors);

    let symbols = Collector::new(&source).collect(root);
    file_map(path, language, source, symbols, parse_errors)
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
            self.collect_declaration(child, prefix, parent_key, symbols);
        }
    }

    fn collect_declaration(
        &mut self,
        node: Node<'_>,
        prefix: &str,
        parent_key: Option<&str>,
        symbols: &mut Vec<Symbol>,
    ) {
        match node.kind() {
            "abstract_class_declaration" | "class_declaration" => {
                if let Some(symbol) = self.class_symbol(node, node, prefix, parent_key) {
                    symbols.push(symbol);
                }
            }
            "function_declaration" | "function_signature" | "generator_function_declaration" => {
                if let Some(symbol) = self.named_function_symbol(node, prefix, parent_key) {
                    symbols.push(symbol);
                }
            }
            "interface_declaration" => {
                if let Some(symbol) = self.interface_symbol(node, prefix, parent_key) {
                    symbols.push(symbol);
                }
            }
            "type_alias_declaration" => {
                if let Some(symbol) = self.type_alias_symbol(node, prefix, parent_key) {
                    symbols.push(symbol);
                }
            }
            "lexical_declaration" | "variable_declaration" => {
                self.collect_variable_declarations(node, prefix, parent_key, symbols);
            }
            "ambient_declaration" | "export_statement" => {
                self.collect_wrapped_declaration(node, prefix, parent_key, symbols);
            }
            _ => {}
        }
    }

    fn collect_wrapped_declaration(
        &mut self,
        node: Node<'_>,
        prefix: &str,
        parent_key: Option<&str>,
        symbols: &mut Vec<Symbol>,
    ) {
        if let Some(declaration) = node.child_by_field_name("declaration") {
            self.collect_declaration(declaration, prefix, parent_key, symbols);
            return;
        }

        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            self.collect_declaration(child, prefix, parent_key, symbols);
        }
    }

    fn collect_variable_declarations(
        &mut self,
        node: Node<'_>,
        prefix: &str,
        parent_key: Option<&str>,
        symbols: &mut Vec<Symbol>,
    ) {
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            if child.kind() != "variable_declarator" {
                continue;
            }
            if let Some(symbol) = self.variable_symbol(child, prefix, parent_key) {
                symbols.push(symbol);
            }
        }
    }

    fn class_symbol(
        &mut self,
        outer: Node<'_>,
        node: Node<'_>,
        prefix: &str,
        parent_key: Option<&str>,
    ) -> Option<Symbol> {
        let name = self.node_field_name(node, "name")?;
        let key_name = sanitize_key(&name);
        let base_key = prefixed_key(prefix, &key_name);
        let key = self.unique_key(&base_key);
        let mut symbol = self.item_symbol(outer, node, SymbolKind::Class, &name, &key, parent_key);

        if let Some(body) = node.child_by_field_name("body") {
            symbol.children = self.class_children(body, &key);
        }

        Some(symbol)
    }

    fn named_function_symbol(
        &mut self,
        node: Node<'_>,
        prefix: &str,
        parent_key: Option<&str>,
    ) -> Option<Symbol> {
        let name = self.node_field_name(node, "name")?;
        let key_name = sanitize_key(&name);
        let base_key = prefixed_key(prefix, &key_name);
        let key = self.unique_key(&base_key);
        Some(self.item_symbol(node, node, SymbolKind::Function, &name, &key, parent_key))
    }

    fn variable_symbol(
        &mut self,
        node: Node<'_>,
        prefix: &str,
        parent_key: Option<&str>,
    ) -> Option<Symbol> {
        let value = node.child_by_field_name("value")?;
        match value.kind() {
            "arrow_function" | "function_expression" | "generator_function" => {
                let name = self.node_field_name(node, "name")?;
                let key_name = sanitize_key(&name);
                let base_key = prefixed_key(prefix, &key_name);
                let key = self.unique_key(&base_key);
                Some(self.item_symbol(node, value, SymbolKind::Function, &name, &key, parent_key))
            }
            "class" => self.variable_class_symbol(node, value, prefix, parent_key),
            _ => None,
        }
    }

    fn variable_class_symbol(
        &mut self,
        outer: Node<'_>,
        node: Node<'_>,
        prefix: &str,
        parent_key: Option<&str>,
    ) -> Option<Symbol> {
        let name = self.node_field_name(outer, "name")?;
        let key_name = sanitize_key(&name);
        let base_key = prefixed_key(prefix, &key_name);
        let key = self.unique_key(&base_key);
        let mut symbol = self.item_symbol(outer, node, SymbolKind::Class, &name, &key, parent_key);

        if let Some(body) = node.child_by_field_name("body") {
            symbol.children = self.class_children(body, &key);
        }

        Some(symbol)
    }

    fn interface_symbol(
        &mut self,
        node: Node<'_>,
        prefix: &str,
        parent_key: Option<&str>,
    ) -> Option<Symbol> {
        let name = self.node_field_name(node, "name")?;
        let key_name = sanitize_key(&name);
        let base_key = prefixed_key(prefix, &key_name);
        let key = self.unique_key(&base_key);
        let mut symbol =
            self.item_symbol(node, node, SymbolKind::Interface, &name, &key, parent_key);

        if let Some(body) = node.child_by_field_name("body") {
            symbol.children = self.interface_children(body, &key);
        }

        Some(symbol)
    }

    fn type_alias_symbol(
        &mut self,
        node: Node<'_>,
        prefix: &str,
        parent_key: Option<&str>,
    ) -> Option<Symbol> {
        let name = self.node_field_name(node, "name")?;
        let key_name = sanitize_key(&name);
        let base_key = prefixed_key(prefix, &key_name);
        let key = self.unique_key(&base_key);
        let mut symbol = self.item_symbol(node, node, SymbolKind::Type, &name, &key, parent_key);

        if let Some(value) = node.child_by_field_name("value") {
            if value.kind() == "object_type" {
                symbol.children = self.interface_children(value, &key);
            }
        }

        Some(symbol)
    }

    fn class_children(&mut self, body: Node<'_>, parent_key: &str) -> Vec<Symbol> {
        let mut children = Vec::new();
        let mut cursor = body.walk();
        for child in body.named_children(&mut cursor) {
            match child.kind() {
                "abstract_method_signature" | "method_definition" | "method_signature" => {
                    if let Some(symbol) = self.method_symbol(child, parent_key) {
                        children.push(symbol);
                    }
                }
                "field_definition" | "public_field_definition" => {
                    if let Some(symbol) = self.class_field_function_symbol(child, parent_key) {
                        children.push(symbol);
                    }
                }
                _ => {}
            }
        }
        children
    }

    fn interface_children(&mut self, body: Node<'_>, parent_key: &str) -> Vec<Symbol> {
        let mut children = Vec::new();
        let mut cursor = body.walk();
        for child in body.named_children(&mut cursor) {
            match child.kind() {
                "abstract_method_signature" | "method_signature" => {
                    if let Some(symbol) = self.method_symbol(child, parent_key) {
                        children.push(symbol);
                    }
                }
                "property_signature" => {
                    if let Some(symbol) = self.property_symbol(child, parent_key) {
                        children.push(symbol);
                    }
                }
                _ => {}
            }
        }
        children
    }

    fn method_symbol(&mut self, node: Node<'_>, parent_key: &str) -> Option<Symbol> {
        let name = self.node_field_name(node, "name")?;
        let key_name = sanitize_key(&name);
        let key = self.unique_key(&format!("{parent_key}.{key_name}"));
        Some(self.item_symbol(
            node,
            node,
            SymbolKind::Method,
            &name,
            &key,
            Some(parent_key),
        ))
    }

    fn class_field_function_symbol(&mut self, node: Node<'_>, parent_key: &str) -> Option<Symbol> {
        let value = node.child_by_field_name("value")?;
        if !is_function_value(value) {
            return None;
        }

        let name = self
            .node_field_name(node, "name")
            .or_else(|| self.node_field_name(node, "property"))?;
        let key_name = sanitize_key(&name);
        let key = self.unique_key(&format!("{parent_key}.{key_name}"));
        Some(self.item_symbol(
            node,
            value,
            SymbolKind::Method,
            &name,
            &key,
            Some(parent_key),
        ))
    }

    fn property_symbol(&mut self, node: Node<'_>, parent_key: &str) -> Option<Symbol> {
        let name = self.node_field_name(node, "name")?;
        let key_name = sanitize_key(&name);
        let key = self.unique_key(&format!("{parent_key}.{key_name}"));
        Some(self.item_symbol(node, node, SymbolKind::Field, &name, &key, Some(parent_key)))
    }

    fn item_symbol(
        &self,
        outer: Node<'_>,
        node: Node<'_>,
        kind: SymbolKind,
        name: &str,
        key: &str,
        parent_key: Option<&str>,
    ) -> Symbol {
        let mut symbol = Symbol::new(
            key,
            kind,
            name,
            self.signature(outer, node),
            line_span(outer),
        );
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

    fn signature(&self, outer: Node<'_>, node: Node<'_>) -> String {
        let start = signature_start_node(outer);
        let end_byte = node
            .child_by_field_name("body")
            .map_or_else(|| outer.end_byte(), |body| body.start_byte());
        let text = &self.source[start.start_byte()..end_byte];
        collapse_whitespace(text.trim().trim_end_matches(';'))
    }

    fn node_field_name(&self, node: Node<'_>, field_name: &str) -> Option<String> {
        node.child_by_field_name(field_name)
            .map(|field| clean_name(self.node_text(field)))
    }

    fn node_text(&self, node: Node<'_>) -> &'a str {
        node.utf8_text(self.source.as_bytes()).unwrap_or("")
    }
}

fn is_function_value(node: Node<'_>) -> bool {
    matches!(
        node.kind(),
        "arrow_function" | "function_expression" | "generator_function"
    )
}

fn signature_start_node<'tree>(node: Node<'tree>) -> Node<'tree> {
    let mut start = node;
    while let Some(parent) = start.parent() {
        match parent.kind() {
            "ambient_declaration" | "export_statement" => start = parent,
            "lexical_declaration" | "variable_declaration"
                if start.kind() == "variable_declarator"
                    && variable_declarator_count(parent) == 1 =>
            {
                start = parent;
            }
            _ => break,
        }
    }
    start
}

fn variable_declarator_count(node: Node<'_>) -> usize {
    let mut cursor = node.walk();
    node.named_children(&mut cursor)
        .filter(|child| child.kind() == "variable_declarator")
        .count()
}

fn clean_name(text: &str) -> String {
    let text = text.trim();
    for quote in ['"', '\'', '`'] {
        if let Some(inner) = text
            .strip_prefix(quote)
            .and_then(|inner| inner.strip_suffix(quote))
        {
            return inner.to_owned();
        }
    }
    text.to_owned()
}

fn prefixed_key(prefix: &str, name: &str) -> String {
    if prefix.is_empty() {
        name.to_owned()
    } else {
        format!("{prefix}.{name}")
    }
}

fn sanitize_key(key: &str) -> String {
    let key = key
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.') {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>();
    if key.is_empty() {
        "_".to_owned()
    } else {
        key
    }
}

fn collapse_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn line_span(node: Node<'_>) -> LineSpan {
    LineSpan::new(node.start_position().row + 1, node.end_position().row + 1)
}
