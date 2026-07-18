use std::collections::BTreeMap;
use std::path::Path;

use tree_sitter::{Node, Parser};

use crate::grammars;
use crate::model::{FileMap, Language, LineSpan, ParseError, Symbol, SymbolKind};

pub fn parse(path: &Path, source: String) -> FileMap {
    parse_as(path, Language::Svelte, source)
}

// The Svelte grammar accepts plain HTML; HTML mode only changes symbol collection.
pub fn parse_html(path: &Path, source: String) -> FileMap {
    parse_as(path, Language::Html, source)
}

fn parse_as(path: &Path, language: Language, source: String) -> FileMap {
    let mut parser = Parser::new();
    let mut parse_errors = Vec::new();
    let Some(grammar) = grammars::language(language) else {
        parse_errors.push(ParseError {
            line: 1,
            message: format!("failed to load {} grammar", grammar_name(language)),
        });
        return file_map(path, language, source, Vec::new(), parse_errors);
    };

    if let Err(err) = parser.set_language(&grammar) {
        parse_errors.push(ParseError {
            line: 1,
            message: format!("failed to load {} grammar: {err}", grammar_name(language)),
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
    collect_parse_errors(&source, root, &mut parse_errors);
    let symbols = Collector::new(&source, language).collect(root);
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

fn grammar_name(language: Language) -> &'static str {
    match language {
        Language::Html => "HTML",
        Language::Svelte => "Svelte",
        _ => unreachable!("markup parser called for {language}"),
    }
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
    language: Language,
    key_counts: BTreeMap<String, usize>,
}

struct HtmlLabel {
    name: String,
    heading_range: Option<(usize, usize)>,
    meaningful: bool,
}

impl<'a> Collector<'a> {
    fn new(source: &'a str, language: Language) -> Self {
        Self {
            source,
            language,
            key_counts: BTreeMap::new(),
        }
    }

    fn collect(mut self, root: Node<'_>) -> Vec<Symbol> {
        let mut symbols = Vec::new();
        if self.language == Language::Html {
            self.collect_html_container(root, "", None, false, None, &mut symbols);
        } else {
            self.collect_container(root, "", None, &mut symbols);
        }
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

    fn collect_html_container(
        &mut self,
        node: Node<'_>,
        prefix: &str,
        parent_key: Option<&str>,
        has_semantic_parent: bool,
        suppressed_heading: Option<(usize, usize)>,
        symbols: &mut Vec<Symbol>,
    ) {
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            match child.kind() {
                "script_element" | "style_element" => {
                    let name = if child.kind() == "script_element" {
                        "script"
                    } else {
                        "style"
                    };
                    symbols.push(self.html_symbol(
                        child,
                        SymbolKind::Node,
                        name,
                        name,
                        prefix,
                        parent_key,
                    ));
                }
                "element" => {
                    self.collect_html_element(
                        child,
                        prefix,
                        parent_key,
                        has_semantic_parent,
                        suppressed_heading,
                        symbols,
                    );
                }
                _ => self.collect_html_container(
                    child,
                    prefix,
                    parent_key,
                    has_semantic_parent,
                    suppressed_heading,
                    symbols,
                ),
            }
        }
    }

    fn collect_html_element(
        &mut self,
        node: Node<'_>,
        prefix: &str,
        parent_key: Option<&str>,
        has_semantic_parent: bool,
        suppressed_heading: Option<(usize, usize)>,
        symbols: &mut Vec<Symbol>,
    ) {
        let Some(tag) = self
            .element_name(node)
            .map(|name| name.to_ascii_lowercase())
        else {
            return;
        };

        if tag == "title" {
            let name = self
                .static_html_text(node)
                .unwrap_or_else(|| "title".to_owned());
            symbols.push(self.html_symbol(
                node,
                SymbolKind::Node,
                "title",
                &name,
                prefix,
                parent_key,
            ));
            return;
        }

        if matches!(tag.as_str(), "script" | "style") {
            symbols.push(self.html_symbol(node, SymbolKind::Node, &tag, &tag, prefix, parent_key));
            return;
        }

        if is_html_heading(&tag) {
            if suppressed_heading != Some((node.start_byte(), node.end_byte())) {
                if let Some(name) = self.static_html_text(node) {
                    symbols.push(self.html_symbol(
                        node,
                        SymbolKind::Heading,
                        &name,
                        &name,
                        prefix,
                        parent_key,
                    ));
                }
            }
            return;
        }

        if is_html_container(&tag) {
            let label = self.html_label(node, &tag);
            // Nested unlabeled landmarks repeat layout scaffolding without adding
            // a useful navigation key. Sections and articles remain navigable.
            let should_emit = matches!(tag.as_str(), "section" | "article")
                || label.meaningful
                || !has_semantic_parent;
            if should_emit {
                let mut symbol = self.html_symbol(
                    node,
                    SymbolKind::Node,
                    &label.name,
                    &label.name,
                    prefix,
                    parent_key,
                );
                self.collect_html_container(
                    node,
                    &symbol.key,
                    Some(&symbol.key),
                    true,
                    label.heading_range,
                    &mut symbol.children,
                );
                symbols.push(symbol);
            } else {
                self.collect_html_container(
                    node,
                    prefix,
                    parent_key,
                    has_semantic_parent,
                    suppressed_heading,
                    symbols,
                );
            }
            return;
        }

        self.collect_html_container(
            node,
            prefix,
            parent_key,
            has_semantic_parent,
            suppressed_heading,
            symbols,
        );
    }

    fn html_label(&self, node: Node<'_>, tag: &str) -> HtmlLabel {
        for attribute in ["data-screen-label", "aria-label", "id"] {
            if let Some(name) = self.static_html_attribute(node, attribute) {
                return HtmlLabel {
                    name,
                    heading_range: None,
                    meaningful: true,
                };
            }
        }

        if let Some((heading, name)) = self.first_static_html_heading(node) {
            return HtmlLabel {
                name,
                heading_range: Some((heading.start_byte(), heading.end_byte())),
                meaningful: true,
            };
        }

        HtmlLabel {
            name: tag.to_owned(),
            heading_range: None,
            meaningful: false,
        }
    }

    fn static_html_attribute(&self, node: Node<'_>, wanted: &str) -> Option<String> {
        let tag =
            child_of_kind(node, "start_tag").or_else(|| child_of_kind(node, "self_closing_tag"))?;
        let mut cursor = tag.walk();
        for attribute in tag.named_children(&mut cursor) {
            if attribute.kind() != "attribute" {
                continue;
            }
            let Some(name) =
                child_of_kind(attribute, "attribute_name").and_then(|name| self.node_text(name))
            else {
                continue;
            };
            if !name.eq_ignore_ascii_case(wanted) {
                continue;
            }

            let value = child_of_kind(attribute, "quoted_attribute_value")
                .or_else(|| child_of_kind(attribute, "attribute_value"))?;
            if self.has_template_expression(value) {
                return None;
            }
            let raw = self.node_text(value)?;
            let unquoted = raw
                .strip_prefix('"')
                .and_then(|value| value.strip_suffix('"'))
                .or_else(|| {
                    raw.strip_prefix('\'')
                        .and_then(|value| value.strip_suffix('\''))
                })
                .unwrap_or(raw.as_str());
            let value = collapse_whitespace(unquoted);
            return (!value.is_empty()).then_some(value);
        }
        None
    }

    fn first_static_html_heading<'tree>(&self, node: Node<'tree>) -> Option<(Node<'tree>, String)> {
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            if child.kind() == "element" {
                let Some(tag) = self
                    .element_name(child)
                    .map(|name| name.to_ascii_lowercase())
                else {
                    continue;
                };
                if is_html_heading(&tag) {
                    if let Some(name) = self.static_html_text(child) {
                        return Some((child, name));
                    }
                    continue;
                }
                if is_html_container(&tag) {
                    continue;
                }
            }
            if let Some(heading) = self.first_static_html_heading(child) {
                return Some(heading);
            }
        }
        None
    }

    fn static_html_text(&self, node: Node<'_>) -> Option<String> {
        if self.has_dynamic_html_text(node) {
            return None;
        }
        let mut parts = Vec::new();
        self.collect_html_text(node, &mut parts);
        let text = collapse_whitespace(&parts.join(" "));
        (!text.is_empty()).then_some(text)
    }

    fn collect_html_text(&self, node: Node<'_>, parts: &mut Vec<String>) {
        if matches!(node.kind(), "text" | "entity") {
            if let Some(text) = self.node_text(node) {
                parts.push(text);
            }
            return;
        }
        if matches!(node.kind(), "script_element" | "style_element") {
            return;
        }

        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            self.collect_html_text(child, parts);
        }
    }

    fn has_dynamic_html_text(&self, node: Node<'_>) -> bool {
        if matches!(
            node.kind(),
            "expression" | "expression_tag" | "html_tag" | "render_tag"
        ) {
            return true;
        }
        if matches!(node.kind(), "start_tag" | "end_tag" | "self_closing_tag") {
            return false;
        }
        if matches!(node.kind(), "text" | "entity") && self.has_template_expression(node) {
            return true;
        }

        let mut cursor = node.walk();
        let has_dynamic = node
            .named_children(&mut cursor)
            .any(|child| self.has_dynamic_html_text(child));
        has_dynamic
    }

    fn has_template_expression(&self, node: Node<'_>) -> bool {
        let Ok(text) = node.utf8_text(self.source.as_bytes()) else {
            return true;
        };
        // Jinja-style expressions are plain text to the Svelte grammar, so AST
        // node kinds alone cannot keep unstable template values out of keys.
        text.contains('{')
            || text.contains('}')
            || text.contains("<%")
            || text.contains("%>")
            || text.contains("[[")
            || text.contains("]]")
    }

    fn html_symbol(
        &mut self,
        node: Node<'_>,
        kind: SymbolKind,
        key_name: &str,
        name: &str,
        prefix: &str,
        parent_key: Option<&str>,
    ) -> Symbol {
        let key = self.unique_key(&prefixed_key(prefix, key_name));
        let mut symbol = Symbol::new(&key, kind, name, self.signature(node), line_span(node));
        symbol.parent_key = parent_key.map(str::to_owned);
        symbol
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

fn is_html_heading(tag: &str) -> bool {
    matches!(tag, "h1" | "h2" | "h3" | "h4" | "h5" | "h6")
}

fn is_html_container(tag: &str) -> bool {
    matches!(
        tag,
        "main" | "section" | "article" | "nav" | "aside" | "header" | "footer"
    )
}

fn collapse_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn line_span(node: Node<'_>) -> LineSpan {
    LineSpan::new(node.start_position().row + 1, node.end_position().row + 1)
}
