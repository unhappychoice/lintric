use crate::edge::Edge;
use std::collections::BTreeSet;
use std::fmt;

/// Marker introducing an expectation annotation in a fixture.
pub const MARKER: &str = "//~";

const DEPENDS: &str = "depends:";

/// Parse `//~ depends: symbol@line, symbol@line` annotations out of a fixture source.
///
/// The annotation applies to the line it appears on, so an expectation can be attached to any
/// line of a multi-line statement by placing the comment on that line.
pub fn parse_expectations(source: &str) -> Result<BTreeSet<Edge>, ParseError> {
    let edges = source
        .lines()
        .enumerate()
        .filter_map(|(index, line)| annotation_body(line).map(|body| (index + 1, body)))
        .map(|(source_line, body)| parse_annotation(source_line, body))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(edges.into_iter().flatten().collect())
}

/// Failure to understand an annotation. Reported rather than ignored, so that a typo in a
/// fixture cannot silently weaken the expected set.
#[derive(Debug, PartialEq)]
pub struct ParseError {
    pub line: usize,
    pub message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}

impl std::error::Error for ParseError {}

impl ParseError {
    fn new(line: usize, message: impl Into<String>) -> Self {
        Self {
            line,
            message: message.into(),
        }
    }
}

fn annotation_body(line: &str) -> Option<&str> {
    line.split_once(MARKER).map(|(_, rest)| rest.trim())
}

fn parse_annotation(source_line: usize, body: &str) -> Result<Vec<Edge>, ParseError> {
    let list = body.strip_prefix(DEPENDS).ok_or_else(|| {
        ParseError::new(
            source_line,
            format!("expected {MARKER} {DEPENDS} ..., found {MARKER} {body}"),
        )
    })?;

    list.split(',')
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
        .map(|entry| parse_entry(source_line, entry))
        .collect()
}

fn parse_entry(source_line: usize, entry: &str) -> Result<Edge, ParseError> {
    let (symbol, target) = entry.rsplit_once('@').ok_or_else(|| {
        ParseError::new(
            source_line,
            format!("expected symbol@line, found {entry:?}"),
        )
    })?;

    let target_line = target
        .trim()
        .parse()
        .map_err(|_| ParseError::new(source_line, format!("{target:?} is not a line number")))?;

    Ok(Edge::new(source_line, target_line, symbol.trim()))
}
