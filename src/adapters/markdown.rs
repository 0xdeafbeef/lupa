use std::collections::HashMap;
use std::path::Path;

use crate::model::{FileMap, Language, LineSpan, Symbol, SymbolKind};

#[derive(Debug)]
struct Heading {
    level: usize,
    start_line: usize,
    end_line: usize,
    name: String,
    signature: String,
}

#[derive(Clone, Copy)]
struct Fence {
    marker: u8,
    len: usize,
}

pub fn parse(path: &Path, source: String) -> FileMap {
    let line_count = count_lines(&source);
    let mut headings = headings(&source);
    set_end_lines(&mut headings, line_count);

    let mut next = 0;
    let mut key_counts = HashMap::new();
    let symbols = build_symbols(&headings, &mut next, 0, None, &mut key_counts);

    FileMap::new(path.to_path_buf(), Language::Markdown, source, symbols)
}

fn headings(source: &str) -> Vec<Heading> {
    let mut headings = Vec::new();
    let mut fence = None;

    for (idx, line) in source.lines().enumerate() {
        if let Some(open_fence) = fence {
            if closes_fence(line, open_fence) {
                fence = None;
            }
            continue;
        }

        if let Some(open_fence) = opens_fence(line) {
            fence = Some(open_fence);
            continue;
        }

        if let Some((level, name, signature)) = parse_heading(line) {
            let start_line = idx + 1;
            headings.push(Heading {
                level,
                start_line,
                end_line: start_line,
                name,
                signature,
            });
        }
    }

    headings
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

fn build_symbols(
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
        symbol.children = build_symbols(
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

fn parse_heading(line: &str) -> Option<(usize, String, String)> {
    let line = strip_optional_indent(line)?;
    let level = line.bytes().take_while(|byte| *byte == b'#').count();
    if !(1..=6).contains(&level) {
        return None;
    }

    if let Some(byte) = line.as_bytes().get(level) {
        if !matches!(*byte, b' ' | b'\t') {
            return None;
        }
    }

    let raw_heading = &line[level..];
    let heading = strip_closing_hashes(raw_heading).trim_matches([' ', '\t']);
    let marker = "#".repeat(level);
    let signature = if heading.is_empty() {
        marker
    } else {
        format!("{marker} {heading}")
    };

    Some((level, heading.to_owned(), signature))
}

fn strip_closing_hashes(heading: &str) -> &str {
    let trimmed = heading.trim_end_matches([' ', '\t']);
    let hash_start = trimmed.trim_end_matches('#').len();
    if hash_start == trimmed.len() {
        return heading;
    }

    if hash_start > 0 && matches!(trimmed.as_bytes()[hash_start - 1], b' ' | b'\t') {
        &trimmed[..hash_start]
    } else {
        heading
    }
}

fn opens_fence(line: &str) -> Option<Fence> {
    let line = strip_optional_indent(line)?;
    let marker = match line.as_bytes().first().copied() {
        Some(marker @ (b'`' | b'~')) => marker,
        _ => return None,
    };
    let len = line.bytes().take_while(|byte| *byte == marker).count();
    (len >= 3).then_some(Fence { marker, len })
}

fn closes_fence(line: &str, fence: Fence) -> bool {
    let Some(line) = strip_optional_indent(line) else {
        return false;
    };
    let len = line
        .bytes()
        .take_while(|byte| *byte == fence.marker)
        .count();
    if len < fence.len {
        return false;
    }

    line[len..].trim_matches([' ', '\t']).is_empty()
}

fn strip_optional_indent(line: &str) -> Option<&str> {
    let spaces = line.bytes().take_while(|byte| *byte == b' ').count();
    (spaces <= 3).then_some(&line[spaces..])
}

fn count_lines(source: &str) -> usize {
    if source.is_empty() {
        0
    } else {
        source.lines().count()
    }
}
