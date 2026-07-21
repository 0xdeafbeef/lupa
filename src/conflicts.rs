use crate::model::LineSpan;

#[derive(Clone, Copy, PartialEq, Eq)]
enum MarkerKind {
    Start,
    End,
    Add,
    Remove,
    Diff,
    Note,
    Ancestor,
    Separator,
}

#[derive(Clone, Copy)]
struct Marker {
    kind: MarkerKind,
    width: usize,
    has_label: bool,
}

fn parse_marker(line: &str) -> Option<Marker> {
    let marker_byte = *line.as_bytes().first()?;
    let kind = match marker_byte {
        b'<' => MarkerKind::Start,
        b'>' => MarkerKind::End,
        b'+' => MarkerKind::Add,
        b'-' => MarkerKind::Remove,
        b'%' => MarkerKind::Diff,
        b'\\' => MarkerKind::Note,
        b'|' => MarkerKind::Ancestor,
        b'=' => MarkerKind::Separator,
        _ => return None,
    };
    let width = line
        .as_bytes()
        .iter()
        .take_while(|byte| **byte == marker_byte)
        .count();
    if width < 7 {
        return None;
    }
    let suffix = &line.as_bytes()[width..];
    if suffix
        .first()
        .is_some_and(|byte| !byte.is_ascii_whitespace())
    {
        return None;
    }
    Some(Marker {
        kind,
        width,
        has_label: suffix.iter().any(|byte| !byte.is_ascii_whitespace()),
    })
}

fn valid_git_body(lines: &[&str], width: usize) -> bool {
    let mut ancestor_seen = false;
    let mut separator_seen = false;

    for line in lines {
        let Some(marker) = parse_marker(line).filter(|marker| marker.width == width) else {
            continue;
        };
        match marker.kind {
            MarkerKind::Ancestor => {
                if ancestor_seen || separator_seen {
                    return false;
                }
                ancestor_seen = true;
            }
            MarkerKind::Separator => {
                if separator_seen || marker.has_label {
                    return false;
                }
                separator_seen = true;
            }
            MarkerKind::Start | MarkerKind::End => return false,
            MarkerKind::Add | MarkerKind::Remove | MarkerKind::Diff | MarkerKind::Note => {}
        }
    }

    separator_seen
}

fn valid_jj_body(lines: &[&str], width: usize) -> bool {
    let first = lines
        .first()
        .and_then(|line| parse_marker(line))
        .filter(|marker| marker.width == width);
    if !matches!(
        first.map(|marker| marker.kind),
        Some(MarkerKind::Diff | MarkerKind::Remove | MarkerKind::Add)
    ) {
        return false;
    }

    let mut remove_count = 0usize;
    let mut add_count = 0usize;
    let mut diff_payload = false;
    let mut note_eligible = false;

    for line in lines {
        let marker = parse_marker(line).filter(|marker| marker.width == width);
        match marker.map(|marker| marker.kind) {
            Some(MarkerKind::Diff) => {
                remove_count += 1;
                add_count += 1;
                diff_payload = true;
                note_eligible = true;
            }
            Some(MarkerKind::Remove) => {
                remove_count += 1;
                diff_payload = false;
                note_eligible = false;
            }
            Some(MarkerKind::Add) => {
                add_count += 1;
                diff_payload = false;
                note_eligible = false;
            }
            Some(MarkerKind::Note) if note_eligible => {
                note_eligible = false;
            }
            Some(MarkerKind::Note | MarkerKind::Ancestor | MarkerKind::Separator) => {
                return false;
            }
            Some(MarkerKind::Start | MarkerKind::End) => return false,
            None => {
                note_eligible = false;
                if diff_payload
                    && !line.is_empty()
                    && !matches!(line.as_bytes()[0], b'-' | b'+' | b' ')
                {
                    return false;
                }
            }
        }
    }

    remove_count > 0 && add_count == remove_count + 1
}

pub(crate) fn detect_conflict_regions(source: &str) -> Vec<LineSpan> {
    let lines = source.lines().collect::<Vec<_>>();
    let mut regions = Vec::new();
    let mut candidate = None::<(usize, usize, bool)>;

    for (line_index, line) in lines.iter().enumerate() {
        let marker = parse_marker(line);
        if let Some(start) = marker.filter(|marker| marker.kind == MarkerKind::Start) {
            candidate = Some((line_index, start.width, false));
            continue;
        }

        let Some((start_index, width, invalid)) = candidate else {
            continue;
        };
        let Some(end) = marker.filter(|marker| marker.kind == MarkerKind::End) else {
            continue;
        };
        if end.width != width {
            candidate = Some((start_index, width, true));
            continue;
        }

        let body = &lines[start_index + 1..line_index];
        if !invalid && (valid_git_body(body, width) || valid_jj_body(body, width)) {
            regions.push(LineSpan::new(start_index + 1, line_index + 1));
        }
        candidate = None;
    }

    regions
}
