use tree_sitter::Node;

#[derive(Debug, Clone)]
pub enum FieldMatcher {
    Exact(String),
    Any,
}

impl FieldMatcher {
    pub fn matches(&self, field_name: Option<&str>) -> bool {
        match self {
            FieldMatcher::Exact(expected) => field_name == Some(expected),
            FieldMatcher::Any => field_name.is_some(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DefinitionPattern {
    pub parent_node_kind: String,
    pub field_matcher: FieldMatcher,
}

impl DefinitionPattern {
    pub fn new(parent_node_kind: &str, field_name: &str) -> Self {
        Self {
            parent_node_kind: parent_node_kind.to_string(),
            field_matcher: FieldMatcher::Exact(field_name.to_string()),
        }
    }

    pub fn with_any_field(parent_node_kind: &str) -> Self {
        Self {
            parent_node_kind: parent_node_kind.to_string(),
            field_matcher: FieldMatcher::Any,
        }
    }
}

pub struct DefinitionContextChecker {
    patterns: Vec<DefinitionPattern>,
}

impl DefinitionContextChecker {
    pub fn new(patterns: Vec<DefinitionPattern>) -> Self {
        Self { patterns }
    }

    pub fn is_identifier_in_definition_context(&self, node: Node) -> bool {
        let mut current = node.parent();

        while let Some(parent) = current {
            for pattern in &self.patterns {
                if parent.kind() == pattern.parent_node_kind {
                    // Check if the node is within the specified field
                    if self.node_is_in_field(&parent, node, &pattern.field_matcher) {
                        return true;
                    }
                }
            }
            current = parent.parent();
        }

        false
    }

    fn node_is_in_field(&self, parent: &Node, target: Node, field_matcher: &FieldMatcher) -> bool {
        match field_matcher {
            FieldMatcher::Any => {
                // Check if target is anywhere under parent
                Self::is_ancestor_or_self(*parent, target)
            }
            FieldMatcher::Exact(field_name) => {
                // Check if target is within the specific field
                if let Some(field_node) = parent.child_by_field_name(field_name) {
                    Self::is_ancestor_or_self(field_node, target)
                } else {
                    // For exact name matching, check if this node itself is the field
                    parent.child_by_field_name("name") == Some(target)
                }
            }
        }
    }

    fn is_ancestor_or_self(ancestor: Node, descendant: Node) -> bool {
        if ancestor == descendant {
            return true;
        }

        let mut cursor = ancestor.walk();
        for child in ancestor.children(&mut cursor) {
            if Self::is_ancestor_or_self(child, descendant) {
                return true;
            }
        }
        false
    }
}
