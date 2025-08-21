pub fn should_display_node_text(node_kind: &str, text: &str) -> bool {
    matches!(
        node_kind,
        // Identifiers and names
        "identifier" | "type_identifier" | "property_identifier" | 
        "shorthand_property_identifier" | "shorthand_property_identifier_pattern" |
        // Literals
        "string_literal" | "number" | "true" | "false" | "null" | "undefined" |
        // Types
        "predefined_type" |
        // Keywords and modifiers
        "this" | "super" | "accessibility_modifier" | "async" | "static" |
        "readonly" | "abstract" | "const" | "let" | "var"
    ) && !text.trim().is_empty()
}

pub fn should_display_tsx_node_text(node_kind: &str, text: &str) -> bool {
    should_display_node_text(node_kind, text)
        || matches!(
            node_kind,
            // JSX-specific identifiers
            "jsx_identifier" | "jsx_attribute_name" | "jsx_property_identifier"
        ) && !text.trim().is_empty()
}
