use crate::collectors::common::usage_node_collector::UsageNodeCollector;
use crate::models::{Usage, UsageKind};
use tree_sitter::Node;

pub struct TypescriptUsageNodeCollector<'a> {
    source_code: &'a str,
}

impl<'a> TypescriptUsageNodeCollector<'a> {
    pub fn new(source_code: &'a str) -> Self {
        Self { source_code }
    }
}

impl<'a> UsageNodeCollector<'a> for TypescriptUsageNodeCollector<'a> {
    fn extract_node_if_usage(
        &self,
        node: Node<'a>,
        current_scope: &Option<String>,
    ) -> Option<Usage<'a>> {
        let kind = match node.kind() {
            "identifier" => Some(UsageKind::Identifier),
            "call_expression" => Some(UsageKind::CallExpression),
            "property_identifier" => Some(UsageKind::FieldExpression),
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
            "function_declaration" | "class_declaration" | "interface_declaration" | "module" => {
                node.child_by_field_name("name").map(|n| {
                    n.utf8_text(self.source_code.as_bytes())
                        .unwrap()
                        .trim()
                        .to_string()
                })
            }
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
