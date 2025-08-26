use crate::languages::rust::formatter as rust_formatter;
use crate::languages::typescript::formatter as typescript_formatter;
use crate::models::Language;

pub struct AstFormatter<'a> {
    file_content: &'a str,
    language: Language,
}

impl<'a> AstFormatter<'a> {
    pub fn new(file_content: &'a str, language: Language) -> Self {
        Self {
            file_content,
            language,
        }
    }

    pub fn format_node(&self, node: tree_sitter::Node, depth: usize) -> String {
        let indent = "  ".repeat(depth);
        let mut s_expr = String::new();

        let node_text = self.format_node_with_text(node);

        s_expr.push_str(&format!("({node_text}"));

        let mut children_s_exprs = Vec::new();

        // Use a cursor to walk through child nodes with field names
        let mut cursor = node.walk();
        if cursor.goto_first_child() {
            loop {
                let child = cursor.node();
                if child.is_named() {
                    let field_name = cursor.field_name();
                    let child_s_expr = self.format_node(child, depth + 1);

                    if let Some(field) = field_name {
                        children_s_exprs.push(format!("{}: {}", field, child_s_expr));
                    } else {
                        children_s_exprs.push(child_s_expr);
                    }
                }

                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }

        if !children_s_exprs.is_empty() {
            for child_s_expr in children_s_exprs {
                s_expr.push_str(&format!("\n{}{}", "  ".repeat(depth + 1), child_s_expr));
            }
            s_expr.push_str(&format!("\n{indent})"));
        } else {
            s_expr.push(')');
        }

        s_expr
    }

    fn format_node_with_text(&self, node: tree_sitter::Node) -> String {
        let node_kind = node.kind();

        // Try to get text content first - this works for all meaningful text nodes
        if let Ok(text) = node.utf8_text(self.file_content.as_bytes()) {
            let normalized_text = self.normalize_line_endings(text);
            let text_trimmed = normalized_text.trim();

            // Only show text if it's not empty and not just structural characters
            if !text_trimmed.is_empty() && self.should_display_node_text(node_kind, text_trimmed) {
                return format!("{} \"{}\"", node_kind, text_trimmed);
            }
        }

        // For nodes without meaningful text content, check if they need special handling
        if self.should_show_position_info(node_kind) {
            let start_pos = node.start_position();
            let end_pos = node.end_position();
            return format!(
                "{} @{}:{}-{}:{}",
                node_kind,
                start_pos.row + 1,
                start_pos.column + 1,
                end_pos.row + 1,
                end_pos.column + 1
            );
        }

        node_kind.to_string()
    }

    fn should_display_node_text(&self, node_kind: &str, text: &str) -> bool {
        match self.language {
            Language::Rust => rust_formatter::should_display_node_text(node_kind, text),
            Language::TypeScript => typescript_formatter::should_display_node_text(node_kind, text),
            Language::TSX => typescript_formatter::should_display_tsx_node_text(node_kind, text),
        }
    }

    fn should_show_position_info(&self, _node_kind: &str) -> bool {
        // Show position for modifier nodes that might not have text content
        // This is now handled by the language-specific functions above
        false
    }

    fn normalize_line_endings(&self, text: &str) -> String {
        text.replace("\r\n", "\n").replace('\r', "\n")
    }
}
