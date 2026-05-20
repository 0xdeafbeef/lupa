use std::fmt;
use std::path::PathBuf;

use crate::model::{FileMap, Symbol};

const MAX_SIBLINGS: usize = 4;

#[derive(Debug, Clone)]
pub struct ContextHit {
    pub path: PathBuf,
    pub line: usize,
}

pub fn parse_hit(input: &str) -> Option<ContextHit> {
    let input = input.trim();
    if input.is_empty() {
        return None;
    }

    for (idx, ch) in input.char_indices() {
        if ch != ':' {
            continue;
        }
        let path = &input[..idx];
        if path.is_empty() {
            continue;
        }
        let rest = &input[idx + 1..];
        let line = rest.split(':').next().unwrap_or_default();
        if let Some(line) = parse_line(line) {
            return Some(ContextHit {
                path: PathBuf::from(path),
                line,
            });
        }
    }

    None
}

pub fn render_context(file: &FileMap, lines: &[usize], out: &mut impl fmt::Write) -> fmt::Result {
    for error in &file.parse_errors {
        writeln!(
            out,
            "# warning: parse error at L{}: {}",
            error.line, error.message
        )?;
    }

    let symbols = file.all_symbols();
    let mut groups = Vec::<ContextGroup<'_>>::new();
    for &line in lines {
        let symbol = deepest_symbol(&symbols, line);
        if let Some(group) = groups
            .iter_mut()
            .find(|group| same_context_group(group.symbol, group.lines[0], symbol, line))
        {
            if !group.lines.contains(&line) {
                group.lines.push(line);
            }
            continue;
        }
        groups.push(ContextGroup {
            symbol,
            lines: vec![line],
        });
    }

    for group in groups {
        match group.symbol {
            Some(symbol) => render_symbol_context(file, symbol, &group.lines, &symbols, out)?,
            None => render_no_symbol_context(file, &group.lines, out)?,
        }
    }

    Ok(())
}

struct ContextGroup<'a> {
    symbol: Option<&'a Symbol>,
    lines: Vec<usize>,
}

fn parse_line(line: &str) -> Option<usize> {
    let line = line.strip_prefix('L').unwrap_or(line);
    line.parse::<usize>().ok().filter(|line| *line > 0)
}

fn deepest_symbol<'a>(symbols: &[&'a Symbol], line: usize) -> Option<&'a Symbol> {
    symbols
        .iter()
        .copied()
        .filter(|symbol| symbol.range.start_line <= line && line <= symbol.range.end_line)
        .min_by(|left, right| {
            let left_len = left.range.end_line - left.range.start_line;
            let right_len = right.range.end_line - right.range.start_line;
            left_len
                .cmp(&right_len)
                .then_with(|| right.key.len().cmp(&left.key.len()))
        })
}

fn same_context_group(
    left: Option<&Symbol>,
    left_line: usize,
    right: Option<&Symbol>,
    right_line: usize,
) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => left.key == right.key,
        (None, None) => left_line == right_line,
        _ => false,
    }
}

fn render_symbol_context(
    file: &FileMap,
    symbol: &Symbol,
    lines: &[usize],
    symbols: &[&Symbol],
    out: &mut impl fmt::Write,
) -> fmt::Result {
    writeln!(
        out,
        "{} {}@{} hits {} {}",
        file.path.display(),
        symbol.key,
        symbol.range,
        format_lines(lines),
        symbol.signature
    )?;

    if let Some(parent) = parent_symbol(symbol, symbols) {
        writeln!(
            out,
            "  parent {}@{} {}",
            parent.key, parent.range, parent.signature
        )?;
    }

    let siblings = sibling_symbols(file, symbol, symbols);
    if !siblings.is_empty() {
        write!(out, "  siblings")?;
        for sibling in siblings.iter().take(MAX_SIBLINGS) {
            write!(out, " {}@{}", sibling.key, sibling.range)?;
        }
        if siblings.len() > MAX_SIBLINGS {
            write!(out, " +{}", siblings.len() - MAX_SIBLINGS)?;
        }
        out.write_char('\n')?;
    }

    Ok(())
}

fn render_no_symbol_context(
    file: &FileMap,
    lines: &[usize],
    out: &mut impl fmt::Write,
) -> fmt::Result {
    writeln!(
        out,
        "{} no-symbol hits {}",
        file.path.display(),
        format_lines(lines)
    )?;

    let min_line = lines.iter().copied().min().unwrap_or(1);
    let max_line = lines.iter().copied().max().unwrap_or(min_line);
    let before = file
        .symbols
        .iter()
        .rev()
        .find(|symbol| symbol.range.end_line < min_line);
    let after = file
        .symbols
        .iter()
        .find(|symbol| symbol.range.start_line > max_line);

    if before.is_some() || after.is_some() {
        write!(out, "  nearby")?;
        if let Some(symbol) = before {
            write!(out, " before {}@{}", symbol.key, symbol.range)?;
        }
        if let Some(symbol) = after {
            write!(out, " after {}@{}", symbol.key, symbol.range)?;
        }
        out.write_char('\n')?;
    }

    Ok(())
}

fn parent_symbol<'a>(symbol: &Symbol, symbols: &[&'a Symbol]) -> Option<&'a Symbol> {
    let parent_key = symbol.parent_key.as_ref()?;
    symbols
        .iter()
        .copied()
        .find(|candidate| candidate.key == *parent_key)
}

fn sibling_symbols<'a>(
    file: &'a FileMap,
    symbol: &Symbol,
    symbols: &[&'a Symbol],
) -> Vec<&'a Symbol> {
    match &symbol.parent_key {
        Some(parent_key) => symbols
            .iter()
            .copied()
            .filter(|candidate| {
                candidate.parent_key.as_ref() == Some(parent_key) && candidate.key != symbol.key
            })
            .collect(),
        None => file
            .symbols
            .iter()
            .filter(|candidate| candidate.key != symbol.key)
            .collect(),
    }
}

fn format_lines(lines: &[usize]) -> String {
    let mut lines = lines.to_vec();
    lines.sort_unstable();
    lines.dedup();
    lines
        .into_iter()
        .map(|line| format!("L{line}"))
        .collect::<Vec<_>>()
        .join(",")
}
