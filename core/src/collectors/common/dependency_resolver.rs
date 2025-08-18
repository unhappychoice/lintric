use crate::models::{Definition, Dependency, DependencyType, Usage, UsageKind};

pub trait DependencyResolver<'a>: Send + Sync {
    fn resolve_dependencies(
        &self,
        source_code: &'a str,
        usage_nodes: &[Usage<'a>],
        definitions: &[Definition],
    ) -> Result<Vec<Dependency>, String>;

    fn resolve_single_dependency(
        &self,
        source_code: &'a str,
        usage_node: &Usage<'a>,
        definitions: &[Definition],
    ) -> Vec<Dependency>;

    fn get_dependency_type(&self, usage_node: &Usage<'a>) -> DependencyType {
        match usage_node.kind {
            UsageKind::Identifier => {
                let parent_kind = usage_node.node.parent().map(|p| p.kind());
                match parent_kind {
                    Some("call_expression") => DependencyType::FunctionCall,
                    Some("macro_invocation") => DependencyType::MacroInvocation,
                    _ => DependencyType::VariableUse,
                }
            }
            UsageKind::CallExpression => DependencyType::FunctionCall,
            UsageKind::FieldExpression => DependencyType::StructFieldAccess,
            UsageKind::StructExpression => DependencyType::TypeReference,
            UsageKind::Metavariable => DependencyType::VariableUse,
        }
    }

    fn get_context(&self, usage_node: &Usage<'a>) -> Option<String> {
        match usage_node.kind {
            UsageKind::CallExpression => Some("call_expression".to_string()),
            UsageKind::FieldExpression => Some("field_access".to_string()),
            UsageKind::StructExpression => Some("struct_instantiation".to_string()),
            UsageKind::Metavariable => Some("metavariable_use".to_string()),
            UsageKind::Identifier => usage_node.node.parent().map(|p| p.kind().to_string()),
        }
    }

    fn add_dependency_if_needed(
        &self,
        dependencies: &mut Vec<Dependency>,
        source_code: &'a str,
        usage_node: &Usage<'a>,
        definitions: &[Definition],
    ) {
        let source_line = usage_node.node.start_position().row + 1;
        let symbol = usage_node
            .node
            .utf8_text(source_code.as_bytes())
            .unwrap()
            .trim()
            .to_string();

        if let Some(def) = find_definition_in_scope(definitions, &symbol, &usage_node.scope) {
            let target_line = def.line_number;
            if source_line != target_line {
                dependencies.push(Dependency {
                    source_line,
                    target_line,
                    symbol,
                    dependency_type: self.get_dependency_type(usage_node),
                    context: self.get_context(usage_node),
                });
            }
        }
    }
}

// Helper function to find a definition considering scope
pub fn find_definition_in_scope<'a>(
    definitions: &'a [Definition],
    name: &str,
    current_scope: &Option<String>,
) -> Option<&'a Definition> {
    // First, try to find a definition that exactly matches the current scope
    if let Some(def) = definitions
        .iter()
        .find(|d| d.name == name && d.scope == *current_scope)
    {
        return Some(def);
    }

    // If current scope is Some, traverse up the ancestor scopes
    if let Some(current_scope_str) = current_scope {
        let mut parts: Vec<&str> = current_scope_str.split('.').collect();
        while !parts.is_empty() {
            parts.pop(); // Remove the innermost scope
            let ancestor_scope = if parts.is_empty() {
                None
            } else {
                Some(parts.join("."))
            };
            if let Some(def) = definitions
                .iter()
                .find(|d| d.name == name && d.scope == ancestor_scope)
            {
                return Some(def);
            }
        }
    }

    // If not found in any specific scope, look for global definitions (scope is None)
    definitions
        .iter()
        .find(|d| d.name == name && d.scope.is_none())
}
