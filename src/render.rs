use std::fmt;

use crate::model::{FileMap, Symbol};

const DIGEST_MAX_FILES: usize = 80;
const DIGEST_MAX_TOP: usize = 12;
const DIGEST_MAX_CHILDREN: usize = 8;

pub fn render_map(file: &FileMap, out: &mut impl fmt::Write) -> fmt::Result {
    let symbols = file.all_symbols();
    writeln!(
        out,
        "# {} [{}] {}L {}B {}S",
        file.path.display(),
        file.language,
        file.line_count,
        file.byte_count,
        symbols.len()
    )?;
    for error in &file.parse_errors {
        writeln!(
            out,
            "# warning: parse error at L{}: {}",
            error.line, error.message
        )?;
    }
    for symbol in &file.symbols {
        render_symbol(symbol, 0, out)?;
    }
    Ok(())
}

pub fn render_keys(file: &FileMap, out: &mut impl fmt::Write) -> fmt::Result {
    for symbol in file.all_symbols() {
        writeln!(out, "{} {}", symbol.key, symbol.range)?;
    }
    Ok(())
}

pub fn render_show(file: &FileMap, keys: &[String], out: &mut impl fmt::Write) -> fmt::Result {
    let symbols = file.all_symbols();
    for key in keys {
        let matches = matching_symbols(&symbols, key);
        match matches.as_slice() {
            [] => render_no_match(file, key, &symbols, out)?,
            [symbol] => render_show_symbol(file, symbol, out)?,
            many => render_ambiguous(file, key, many, out)?,
        }
    }
    Ok(())
}

pub fn render_digest(files: &[FileMap], out: &mut impl fmt::Write) -> fmt::Result {
    if files.len() > DIGEST_MAX_FILES {
        writeln!(out, "# truncated files {DIGEST_MAX_FILES}/{}", files.len())?;
    }
    for file in files.iter().take(DIGEST_MAX_FILES) {
        write!(
            out,
            "{} [{}] {}L {}S",
            file.path.display(),
            file.language,
            file.line_count,
            file.all_symbols().len()
        )?;
        if !file.parse_errors.is_empty() {
            write!(out, " {}E", file.parse_errors.len())?;
        }
        for symbol in file.symbols.iter().take(DIGEST_MAX_TOP) {
            write!(out, " {}@{}", symbol.key, symbol.range)?;
            let prefix = format!("{}.", symbol.key);
            let children = symbol
                .children
                .iter()
                .take(DIGEST_MAX_CHILDREN)
                .map(|child| {
                    child
                        .key
                        .strip_prefix(&prefix)
                        .unwrap_or(child.key.as_str())
                })
                .collect::<Vec<_>>();
            if !children.is_empty() {
                write!(out, "[{}]", children.join(","))?;
                if symbol.children.len() > DIGEST_MAX_CHILDREN {
                    write!(out, "[+{}]", symbol.children.len() - DIGEST_MAX_CHILDREN)?;
                }
            }
        }
        if file.symbols.len() > DIGEST_MAX_TOP {
            write!(out, " +{}", file.symbols.len() - DIGEST_MAX_TOP)?;
        }
        out.write_char('\n')?;
    }
    Ok(())
}

fn render_symbol(symbol: &Symbol, depth: usize, out: &mut impl fmt::Write) -> fmt::Result {
    let indent = "  ".repeat(depth);
    writeln!(
        out,
        "{}{} {} {}",
        indent, symbol.range, symbol.key, symbol.signature
    )?;
    for child in &symbol.children {
        render_symbol(child, depth + 1, out)?;
    }
    Ok(())
}

fn matching_symbols<'a>(symbols: &'a [&Symbol], key: &str) -> Vec<&'a Symbol> {
    let exact = symbols
        .iter()
        .copied()
        .filter(|symbol| symbol.key == key)
        .collect::<Vec<_>>();
    if !exact.is_empty() {
        return exact;
    }
    symbols
        .iter()
        .copied()
        .filter(|symbol| symbol.key.ends_with(&format!(".{key}")))
        .collect()
}

fn render_no_match(
    _file: &FileMap,
    key: &str,
    symbols: &[&Symbol],
    out: &mut impl fmt::Write,
) -> fmt::Result {
    writeln!(out, "# no {key}")?;
    let candidates = symbols
        .iter()
        .copied()
        .filter(|symbol| symbol.key.contains(key) || symbol.name.contains(key))
        .take(8)
        .collect::<Vec<_>>();
    if !candidates.is_empty() {
        writeln!(out, "# candidates")?;
        for symbol in candidates {
            writeln!(
                out,
                "# {}@{} {}",
                symbol.key, symbol.range, symbol.signature
            )?;
        }
    }
    Ok(())
}

fn render_ambiguous(
    _file: &FileMap,
    key: &str,
    symbols: &[&Symbol],
    out: &mut impl fmt::Write,
) -> fmt::Result {
    writeln!(out, "# amb {key}")?;
    for symbol in symbols {
        writeln!(
            out,
            "# {}@{} {}",
            symbol.key, symbol.range, symbol.signature
        )?;
    }
    Ok(())
}

fn render_show_symbol(file: &FileMap, symbol: &Symbol, out: &mut impl fmt::Write) -> fmt::Result {
    writeln!(out, "# {}@{}", symbol.key, symbol.range)?;
    let lines = file
        .source
        .lines()
        .enumerate()
        .filter_map(|(idx, line)| {
            let line_no = idx + 1;
            (symbol.range.start_line..=symbol.range.end_line)
                .contains(&line_no)
                .then_some((line_no, line))
        })
        .collect::<Vec<_>>();
    let trim = common_indent(&lines);
    for (_, line) in lines {
        writeln!(out, "{}", trim_indent(line, trim))?;
    }
    Ok(())
}

fn common_indent(lines: &[(usize, &str)]) -> usize {
    lines
        .iter()
        .map(|(_, line)| line)
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            line.char_indices()
                .find_map(|(idx, ch)| (!ch.is_whitespace()).then_some(idx))
                .unwrap_or(line.len())
        })
        .min()
        .unwrap_or(0)
}

fn trim_indent(line: &str, trim: usize) -> &str {
    if line.len() <= trim {
        return line.trim_start();
    }
    line.get(trim..).unwrap_or_else(|| line.trim_start())
}
