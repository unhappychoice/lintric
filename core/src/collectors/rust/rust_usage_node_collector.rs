use crate::collectors::common::definition_context::{DefinitionContextChecker, DefinitionPattern};
use crate::collectors::common::usage_node_collector::UsageNodeCollector;
use crate::models::{Usage, UsageKind};
use tree_sitter::Node;

pub struct RustUsageNodeCollector<'a> {
    source_code: &'a str,
    definition_checker: DefinitionContextChecker,
}

impl<'a> RustUsageNodeCollector<'a> {
    pub fn new(source_code: &'a str) -> Self {
        let patterns = vec![
            // Variable declarations
            DefinitionPattern::new("let_declaration", "pattern"),
            DefinitionPattern::new("parameter", "pattern"),
            DefinitionPattern::new("for_expression", "pattern"),
            DefinitionPattern::with_any_field("closure_parameters"),
            // Named items
            DefinitionPattern::new("function_item", "name"),
            DefinitionPattern::new("struct_item", "name"),
            DefinitionPattern::new("enum_item", "name"),
            DefinitionPattern::new("trait_item", "name"),
            DefinitionPattern::new("mod_item", "name"),
            DefinitionPattern::new("const_item", "name"),
            DefinitionPattern::new("static_item", "name"),
        ];

        Self {
            source_code,
            definition_checker: DefinitionContextChecker::new(patterns),
        }
    }
}

impl<'a> UsageNodeCollector<'a> for RustUsageNodeCollector<'a> {
    fn extract_node_if_usage(
        &self,
        node: Node<'a>,
        current_scope: &Option<String>,
    ) -> Option<Usage<'a>> {
        let kind = match node.kind() {
            "identifier" => {
                // Only treat identifier as usage if it's not in a definition context
                if self
                    .definition_checker
                    .is_identifier_in_definition_context(node)
                {
                    None
                } else {
                    Some(UsageKind::Identifier)
                }
            }
            "call_expression" => Some(UsageKind::CallExpression),
            "field_expression" => Some(UsageKind::FieldExpression),
            "struct_expression" => Some(UsageKind::StructExpression),
            "metavariable" => Some(UsageKind::Metavariable),
            _ => None,
        };

        kind.map(|k| Usage {
            node,
            kind: k,
            scope: current_scope.clone(),
        })
    }

    fn determine_scope(&self, node: &Node<'a>, parent_scope: &Option<String>) -> Option<String> {
        let new_scope_name = match node.kind() {
            "function_item" | "struct_item" | "enum_item" | "trait_item" => {
                node.child_by_field_name("name").map(|n| {
                    n.utf8_text(self.source_code.as_bytes())
                        .unwrap()
                        .trim()
                        .to_string()
                })
            }
            "impl_item" => node.child_by_field_name("type").map(|n| {
                n.utf8_text(self.source_code.as_bytes())
                    .unwrap()
                    .trim()
                    .to_string()
            }),
            _ => None,
        };

        if let Some(name) = new_scope_name {
            Some(
                parent_scope
                    .as_ref()
                    .map_or(name.clone(), |p| format!("{p}.{name}")),
            )
        } else {
            parent_scope.clone()
        }
    }
}
