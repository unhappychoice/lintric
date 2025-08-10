use crate::parsers::{rust_parser, typescript_parser};
use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;

pub fn parse_code(
    content: &str,
    file_path: &str,
) -> Result<(DiGraph<usize, usize>, HashMap<usize, NodeIndex>), String> {
    if file_path.ends_with(".rs") {
        rust_parser::parse_rust_code(content)
    } else if file_path.ends_with(".ts") {
        typescript_parser::parse_typescript_code(content, false)
    } else if file_path.ends_with(".tsx") {
        typescript_parser::parse_typescript_code(content, true)
    } else {
        Err(format!(
            "Unsupported file extension for {}. Only .rs, .ts, .tsx are supported.",
            file_path
        ))
    }
}
