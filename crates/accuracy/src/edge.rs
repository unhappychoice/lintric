use serde::{Deserialize, Serialize};
use std::fmt;

/// A single line-to-line dependency, identified by source line, target line and symbol.
///
/// This is the unit of comparison between hand-written expectations and analyzer output.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Edge {
    pub source_line: usize,
    pub target_line: usize,
    pub symbol: String,
}

impl Edge {
    pub fn new(source_line: usize, target_line: usize, symbol: impl Into<String>) -> Self {
        Self {
            source_line,
            target_line,
            symbol: symbol.into(),
        }
    }
}

impl fmt::Display for Edge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "L{} -> L{} {:?}",
            self.source_line, self.target_line, self.symbol
        )
    }
}
