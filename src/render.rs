use std::fmt;

use crate::model::{FileMap, Symbol};

pub fn render_map(file: &FileMap, out: &mut impl fmt::Write) -> fmt::Result {
    let symbols = file.all_symbols();
    writeln!(
        out,
        "# {} [{}] {} lines, {} bytes, {} symbols",
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
    for file in files {
        writeln!(
            out,
            "{} [{}] {} lines, {} symbols",
            file.path.display(),
            file.language,
            file.line_count,
            file.all_symbols().len()
        )?;
        let top = file
            .symbols
            .iter()
            .map(|symbol| symbol.key.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        if !top.is_empty() {
            writeln!(out, "  {top}")?;
        }
    }
    Ok(())
}

fn render_symbol(symbol: &Symbol, depth: usize, out: &mut impl fmt::Write) -> fmt::Result {
    let indent = "    ".repeat(depth);
    writeln!(
        out,
        "{}{}  {}  key={}",
        indent, symbol.signature, symbol.range, symbol.key
    )?;
    for child in &symbol.children {
        render_symbol(child, depth + 1, out)?;
    }
    if depth == 0 {
        out.write_char('\n')?;
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
    file: &FileMap,
    key: &str,
    symbols: &[&Symbol],
    out: &mut impl fmt::Write,
) -> fmt::Result {
    writeln!(
        out,
        "# error: no symbol matching `{key}` in {}",
        file.path.display()
    )?;
    let candidates = symbols
        .iter()
        .copied()
        .filter(|symbol| symbol.key.contains(key) || symbol.name.contains(key))
        .take(8)
        .collect::<Vec<_>>();
    if !candidates.is_empty() {
        writeln!(out, "# candidates:")?;
        for symbol in candidates {
            writeln!(
                out,
                "#   {} {} key={}",
                symbol.signature, symbol.range, symbol.key
            )?;
        }
    }
    Ok(())
}

fn render_ambiguous(
    file: &FileMap,
    key: &str,
    symbols: &[&Symbol],
    out: &mut impl fmt::Write,
) -> fmt::Result {
    writeln!(
        out,
        "# error: ambiguous key `{key}` in {}",
        file.path.display()
    )?;
    writeln!(out, "# matches:")?;
    for symbol in symbols {
        writeln!(
            out,
            "#   {} {} key={}",
            symbol.signature, symbol.range, symbol.key
        )?;
    }
    Ok(())
}

fn render_show_symbol(file: &FileMap, symbol: &Symbol, out: &mut impl fmt::Write) -> fmt::Result {
    writeln!(
        out,
        "# {} {} key={} kind={}",
        file.path.display(),
        symbol.range,
        symbol.key,
        symbol.kind
    )?;
    if let Some(parent) = &symbol.parent_key {
        writeln!(out, "# in: {parent}")?;
    }
    for (idx, line) in file.source.lines().enumerate() {
        let line_no = idx + 1;
        if (symbol.range.start_line..=symbol.range.end_line).contains(&line_no) {
            writeln!(out, "{line_no}|{line}")?;
        }
    }
    Ok(())
}
