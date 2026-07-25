use crate::models::{Position, ScopeId, Usage, UsageKind};
use tree_sitter::Node;

/// Macros whose first string argument is a format string.
const FORMAT_MACROS: [&str; 18] = [
    "assert",
    "assert_eq",
    "assert_ne",
    "debug_assert",
    "debug_assert_eq",
    "debug_assert_ne",
    "eprint",
    "eprintln",
    "format",
    "format_args",
    "panic",
    "print",
    "println",
    "todo",
    "unimplemented",
    "unreachable",
    "write",
    "writeln",
];

/// Extract the usages a format string captures inline.
///
/// Since Rust 2021, `println!("{name}")` refers to `name` directly rather than through the
/// argument list, so the identifier only exists inside the string literal and is invisible to
/// AST traversal. `node` is expected to be a `string_content`.
pub fn capture_usages(node: Node, scope: ScopeId, source: &str) -> Vec<Usage> {
    match is_format_string(node, source) {
        false => vec![],
        true => node
            .utf8_text(source.as_bytes())
            .map(|content| usages(&node, scope, content))
            .unwrap_or_default(),
    }
}

/// An identifier a format string captures, as in `"{name}"` or `"{value:>width$}"`.
struct Capture {
    name: String,
    /// Byte offset of the identifier within the format string content.
    offset: usize,
}

fn usages(node: &Node, scope: ScopeId, content: &str) -> Vec<Usage> {
    captures(content)
        .into_iter()
        .map(|capture| Usage {
            position: position(node, content, &capture),
            name: capture.name,
            kind: UsageKind::Identifier,
            context: Some("format_string".to_string()),
            scope_id: Some(scope),
        })
        .collect()
}

/// Identifiers captured by the placeholders of a format string.
///
/// Positional (`"{0}"`) and empty (`"{}"`) placeholders take their value from the argument list,
/// so they capture nothing.
fn captures(content: &str) -> Vec<Capture> {
    placeholders(content)
        .into_iter()
        .flat_map(|(offset, body)| placeholder_captures(offset, body))
        .collect()
}

/// Byte offset and body of every `{...}` placeholder, skipping `{{` and `}}` escapes.
fn placeholders(content: &str) -> Vec<(usize, &str)> {
    let chars: Vec<(usize, char)> = content.char_indices().collect();
    let mut found = Vec::new();
    let mut index = 0;

    while index < chars.len() {
        let pair = (chars[index].1, chars.get(index + 1).map(|(_, c)| *c));

        if matches!(pair, ('{', Some('{')) | ('}', Some('}'))) {
            index += 2;
            continue;
        }

        if pair.0 != '{' {
            index += 1;
            continue;
        }

        match closing_brace(&chars, index) {
            None => break,
            Some(close) => {
                // `{` is single-byte, so the body starts on a character boundary.
                let start = chars[index].0 + 1;
                found.push((start, &content[start..chars[close].0]));
                index = close + 1;
            }
        }
    }

    found
}

fn closing_brace(chars: &[(usize, char)], open: usize) -> Option<usize> {
    chars
        .iter()
        .enumerate()
        .skip(open + 1)
        .find(|(_, (_, character))| *character == '}')
        .map(|(index, _)| index)
}

fn placeholder_captures(offset: usize, body: &str) -> Vec<Capture> {
    let (argument, spec) = match body.split_once(':') {
        Some((argument, spec)) => (argument, Some(spec)),
        None => (body, None),
    };

    let named = identifier(argument).map(|name| Capture { name, offset });
    let referenced = spec
        .map(|spec| spec_captures(spec, offset + argument.len() + 1))
        .unwrap_or_default();

    named.into_iter().chain(referenced).collect()
}

/// `name$` inside a format spec references a binding for width or precision, as in `"{:w$}"`.
fn spec_captures(spec: &str, offset: usize) -> Vec<Capture> {
    spec.match_indices('$')
        .filter_map(|(end, _)| identifier_ending_at(spec, end))
        .map(|(start, name)| Capture {
            name,
            offset: offset + start,
        })
        .collect()
}

fn identifier_ending_at(spec: &str, end: usize) -> Option<(usize, String)> {
    let start = spec[..end]
        .char_indices()
        .rev()
        .take_while(|(_, character)| is_identifier_char(*character))
        .last()
        .map(|(index, _)| index)?;

    identifier(&spec[start..end]).map(|name| (start, name))
}

fn identifier(text: &str) -> Option<String> {
    let text = text.trim();
    let first = text.chars().next()?;

    (first.is_alphabetic() || first == '_')
        .then(|| text.to_string())
        .filter(|text| text.chars().all(is_identifier_char))
}

fn is_identifier_char(character: char) -> bool {
    character.is_alphanumeric() || character == '_'
}

fn position(node: &Node, content: &str, capture: &Capture) -> Position {
    let start = node.start_position();
    let prefix = &content[..capture.offset];
    let line = start.row + 1 + prefix.matches('\n').count();

    // Columns are byte offsets within their line, so a capture on the string's first line is
    // offset from the string's own column and a later line starts from zero.
    let column = match prefix.rfind('\n') {
        Some(index) => capture.offset - index - 1,
        None => start.column + capture.offset,
    };

    Position {
        start_line: line,
        start_column: column + 1,
        end_line: line,
        end_column: column + 1 + capture.name.len(),
    }
}

fn is_format_string(node: Node, source: &str) -> bool {
    parent_of_kind(node, "string_literal")
        .and_then(|literal| parent_of_kind(literal, "token_tree").map(|tokens| (literal, tokens)))
        .and_then(|(literal, tokens)| {
            parent_of_kind(tokens, "macro_invocation")
                .map(|macro_node| (literal, tokens, macro_node))
        })
        .is_some_and(|(literal, tokens, macro_node)| {
            is_format_macro(&macro_node, source) && is_first_string_literal(&tokens, &literal)
        })
}

fn parent_of_kind<'a>(node: Node<'a>, kind: &str) -> Option<Node<'a>> {
    node.parent().filter(|parent| parent.kind() == kind)
}

fn is_format_macro(macro_node: &Node, source: &str) -> bool {
    macro_node
        .child_by_field_name("macro")
        .and_then(|name| name.utf8_text(source.as_bytes()).ok())
        .is_some_and(|name| FORMAT_MACROS.contains(&name))
}

/// Only the first string in the token tree is the format string; later ones are arguments,
/// so their braces are data rather than placeholders.
fn is_first_string_literal(tokens: &Node, literal: &Node) -> bool {
    let mut cursor = tokens.walk();

    let first = tokens
        .children(&mut cursor)
        .find(|child| child.kind() == "string_literal");

    first.is_some_and(|first| first.id() == literal.id())
}
