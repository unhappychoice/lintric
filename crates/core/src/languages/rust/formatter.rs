pub fn should_display_node_text(node_kind: &str, text: &str) -> bool {
    matches!(
        node_kind,
        // Identifiers and names
        "identifier" | "type_identifier" | "field_identifier" | 
        "scoped_identifier" | "scoped_type_identifier" |
        // Literals
        "string_literal" | "raw_string_literal" | "integer_literal" | 
        "float_literal" | "boolean_literal" | "char_literal" |
        // Types
        "primitive_type" |
        // Keywords that might have text content
        "self" | "super" | "crate"
    ) && !text.trim().is_empty()
}
