use std::collections::BTreeMap;
use std::path::Path;

use tree_sitter::{Language as TsLanguage, Node, Parser};

use crate::model::{FileMap, Language, LineSpan, ParseError, Symbol, SymbolKind};

#[derive(Debug, Clone, Copy)]
enum LanguageVariant {
    C,
    Cpp,
}

impl LanguageVariant {
    fn from_language(language: Language) -> Self {
        match language {
            Language::C => Self::C,
            Language::Cpp => Self::Cpp,
            language => unreachable!("{language} passed to C-family parser"),
        }
    }

    fn language(self) -> TsLanguage {
        match self {
            Self::C => tree_sitter_c::LANGUAGE.into(),
            Self::Cpp => tree_sitter_cpp::LANGUAGE.into(),
        }
    }

    fn grammar_name(self) -> &'static str {
        match self {
            Self::C => "C",
            Self::Cpp => "C++",
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
            line: node.start_position().row + 1,
            message: format!("parse error in {}", node.kind()),
        });
        return;
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
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
        self.collect_container(root, "", &mut symbols);
        symbols
    }

    fn collect_container(&mut self, node: Node<'_>, prefix: &str, symbols: &mut Vec<Symbol>) {
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            self.collect_item(child, child, prefix, None, symbols);
        }
    }

    fn collect_item(
        &mut self,
        outer: Node<'_>,
        node: Node<'_>,
        prefix: &str,
        parent_key: Option<&str>,
        symbols: &mut Vec<Symbol>,
    ) {
        match node.kind() {
            "class_specifier" => {
                if let Some(symbol) = self.type_symbol(outer, node, prefix, SymbolKind::Class) {
                    symbols.push(symbol);
                }
            }
            "declaration" | "field_declaration" => {
                self.collect_declaration(outer, node, prefix, parent_key, symbols);
            }
            "enum_specifier" => {
                if let Some(symbol) = self.type_symbol(outer, node, prefix, SymbolKind::Enum) {
                    symbols.push(symbol);
                }
            }
            "function_definition" => {
                if let Some(symbol) = self.function_symbol(outer, node, prefix, parent_key) {
                    symbols.push(symbol);
                }
            }
            "namespace_definition" => self.collect_namespace(node, prefix, symbols),
            "struct_specifier" | "union_specifier" => {
                if let Some(symbol) = self.type_symbol(outer, node, prefix, SymbolKind::Struct) {
                    symbols.push(symbol);
                }
            }
            "template_declaration" => self.collect_template(node, prefix, parent_key, symbols),
            "type_definition" | "alias_declaration" => {
                if let Some(symbol) = self.alias_symbol(outer, node, prefix, parent_key) {
                    symbols.push(symbol);
                }
            }
            _ => {}
        }
    }

    fn collect_namespace(&mut self, node: Node<'_>, prefix: &str, symbols: &mut Vec<Symbol>) {
        let Some(name) = self.node_field_text(node, "name") else {
            if let Some(body) = node.child_by_field_name("body") {
                self.collect_container(body, prefix, symbols);
            }
            return;
        };

        let key = prefixed_key(prefix, &sanitize_key(name));
        if let Some(body) = node.child_by_field_name("body") {
            self.collect_container(body, &key, symbols);
        }
    }

    fn collect_template(
        &mut self,
        node: Node<'_>,
        prefix: &str,
        parent_key: Option<&str>,
        symbols: &mut Vec<Symbol>,
    ) {
        let Some(inner) = last_named_child(node) else {
            return;
        };
        self.collect_item(node, inner, prefix, parent_key, symbols);
    }

    fn collect_declaration(
        &mut self,
        outer: Node<'_>,
        node: Node<'_>,
        prefix: &str,
        parent_key: Option<&str>,
        symbols: &mut Vec<Symbol>,
    ) {
        if let Some(declarator) = find_descendant(node, "function_declarator") {
            if let Some(symbol) =
                self.function_declarator_symbol(outer, declarator, prefix, parent_key)
            {
                symbols.push(symbol);
            }
            return;
        }

        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            match child.kind() {
                "class_specifier" => {
                    if let Some(symbol) = self.type_symbol(outer, child, prefix, SymbolKind::Class)
                    {
                        symbols.push(symbol);
                    }
                }
                "enum_specifier" => {
                    if let Some(symbol) = self.type_symbol(outer, child, prefix, SymbolKind::Enum) {
                        symbols.push(symbol);
                    }
                }
                "struct_specifier" | "union_specifier" => {
                    if let Some(symbol) = self.type_symbol(outer, child, prefix, SymbolKind::Struct)
                    {
                        symbols.push(symbol);
                    }
                }
                _ => {}
            }
        }
    }

    fn type_symbol(
        &mut self,
        outer: Node<'_>,
        node: Node<'_>,
        prefix: &str,
        kind: SymbolKind,
    ) -> Option<Symbol> {
        let name = self.type_name(node)?;
        let key_name = sanitize_key(&name);
        let base_key = prefixed_key(prefix, &key_name);
        let key = self.unique_key(&base_key);
        let mut symbol = self.item_symbol(outer, node, kind, &name, &key, None);
        if let Some(body) = node.child_by_field_name("body") {
            symbol.children = self.type_children(body, &key);
        }
        Some(symbol)
    }

    fn alias_symbol(
        &mut self,
        outer: Node<'_>,
        node: Node<'_>,
        prefix: &str,
        parent_key: Option<&str>,
    ) -> Option<Symbol> {
        let declarator = node
            .child_by_field_name("declarator")
            .or_else(|| find_descendant(node, "type_identifier"))?;
        let name = declarator_name(declarator, self.source)?;
        let key_name = sanitize_key(&name);
        let base_key = prefixed_key(prefix, &key_name);
        let key = self.unique_key(&base_key);
        let mut symbol = self.item_symbol(outer, node, SymbolKind::Type, &name, &key, parent_key);
        if let Some(specifier) = first_specifier(node) {
            if let Some(body) = specifier.child_by_field_name("body") {
                symbol.children = self.type_children(body, &key);
            }
        }
        Some(symbol)
    }

    fn function_symbol(
        &mut self,
        outer: Node<'_>,
        node: Node<'_>,
        prefix: &str,
        parent_key: Option<&str>,
    ) -> Option<Symbol> {
        let declarator = node.child_by_field_name("declarator")?;
        self.function_declarator_symbol(outer, declarator, prefix, parent_key)
    }

    fn function_declarator_symbol(
        &mut self,
        outer: Node<'_>,
        declarator: Node<'_>,
        prefix: &str,
        parent_key: Option<&str>,
    ) -> Option<Symbol> {
        let name = declarator_name(declarator, self.source)?;
        let key_name = sanitize_key(&name);
        let base_key = parent_key.map_or_else(
            || prefixed_key(prefix, &key_name),
            |parent| format!("{parent}.{key_name}"),
        );
        let key = self.unique_key(&base_key);
        let kind = parent_key.map_or(SymbolKind::Function, |_| SymbolKind::Method);
        Some(self.item_symbol(outer, declarator, kind, &name, &key, parent_key))
    }

    fn type_children(&mut self, body: Node<'_>, parent_key: &str) -> Vec<Symbol> {
        let mut children = Vec::new();
        let mut cursor = body.walk();
        for child in body.named_children(&mut cursor) {
            match child.kind() {
                "declaration" | "field_declaration" => {
                    self.collect_declaration(child, child, "", Some(parent_key), &mut children);
                    if !has_descendant(child, "function_declarator") {
                        self.push_field_symbols(child, parent_key, &mut children);
                    }
                }
                "function_definition" => {
                    if let Some(symbol) = self.function_symbol(child, child, "", Some(parent_key)) {
                        children.push(symbol);
                    }
                }
                "template_declaration" => {
                    self.collect_template(child, "", Some(parent_key), &mut children);
                }
                "type_definition" | "alias_declaration" => {
                    if let Some(symbol) = self.alias_symbol(child, child, "", Some(parent_key)) {
                        children.push(symbol);
                    }
                }
                _ => {}
            }
        }
        children
    }

    fn push_field_symbols(&mut self, node: Node<'_>, parent_key: &str, children: &mut Vec<Symbol>) {
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            if matches!(
                child.kind(),
                "field_identifier" | "identifier" | "pointer_declarator" | "array_declarator"
            ) {
                if let Some(name) = declarator_name(child, self.source) {
                    let key_name = sanitize_key(&name);
                    let key = self.unique_key(&format!("{parent_key}.{key_name}"));
                    children.push(self.item_symbol(
                        node,
                        child,
                        SymbolKind::Field,
                        &name,
                        &key,
                        Some(parent_key),
                    ));
                }
            }
        }
    }

    fn type_name(&self, node: Node<'_>) -> Option<String> {
        node.child_by_field_name("name")
            .and_then(|node| node_text(node, self.source))
            .map(clean_name)
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

    fn signature(&self, outer: Node<'_>, node: Node<'_>) -> String {
        let end_byte = node
            .child_by_field_name("body")
            .map_or_else(|| outer.end_byte(), |body| body.start_byte());
        let text = &self.source[outer.start_byte()..end_byte];
        collapse_whitespace(text.trim().trim_end_matches(';'))
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

    fn node_field_text(&self, node: Node<'_>, field_name: &str) -> Option<&'a str> {
        node.child_by_field_name(field_name)
            .and_then(|field| node_text(field, self.source))
    }
}

fn first_specifier(node: Node<'_>) -> Option<Node<'_>> {
    let mut cursor = node.walk();
    let specifier = node.named_children(&mut cursor).find(|child| {
        matches!(
            child.kind(),
            "class_specifier" | "enum_specifier" | "struct_specifier" | "union_specifier"
        )
    });
    specifier
}

fn find_descendant<'tree>(node: Node<'tree>, kind: &str) -> Option<Node<'tree>> {
    if node.kind() == kind {
        return Some(node);
    }
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        if let Some(found) = find_descendant(child, kind) {
            return Some(found);
        }
    }
    None
}

fn has_descendant(node: Node<'_>, kind: &str) -> bool {
    find_descendant(node, kind).is_some()
}

fn last_named_child(node: Node<'_>) -> Option<Node<'_>> {
    let mut cursor = node.walk();
    node.named_children(&mut cursor).last()
}

fn declarator_name(node: Node<'_>, source: &str) -> Option<String> {
    match node.kind() {
        "destructor_name" | "field_identifier" | "identifier" | "operator_name"
        | "type_identifier" => node_text(node, source).map(clean_name),
        "qualified_identifier" | "template_function" | "template_type" => {
            node_text(node, source).map(qualified_key)
        }
        _ => node
            .child_by_field_name("declarator")
            .and_then(|node| declarator_name(node, source))
            .or_else(|| {
                node.child_by_field_name("name")
                    .and_then(|node| declarator_name(node, source))
            })
            .or_else(|| {
                let mut cursor = node.walk();
                let name = node
                    .named_children(&mut cursor)
                    .find_map(|child| declarator_name(child, source));
                name
            }),
    }
}

fn node_text<'a>(node: Node<'_>, source: &'a str) -> Option<&'a str> {
    node.utf8_text(source.as_bytes()).ok()
}

fn clean_name(text: &str) -> String {
    text.trim().to_owned()
}

fn qualified_key(text: &str) -> String {
    text.replace("::", ".")
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
        .replace("operator==", "operator_eq")
        .replace("operator!=", "operator_ne")
        .replace("operator()", "operator_call")
        .replace("operator[]", "operator_index")
        .replace("operator=", "operator_assign");
    let key = key
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.' | '~') {
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
