use crate::models::{Definition, Dependency, DependencyType};
use tree_sitter::Node;

pub trait DependencyCollector<'a>: Send + Sync {
    fn collect_dependencies_from_root(
        &self,
        root: Node<'a>,
        definitions: &[Definition],
    ) -> Result<Vec<Dependency>, String> {
        let mut dependencies: Vec<Dependency> = Vec::new();
        let mut stack: Vec<(Node<'a>, Option<String>)> = Vec::new();
        stack.push((root, None));

        while let Some((node, current_scope)) = stack.pop() {
            let new_scope = self.determine_scope(&node, &current_scope);

            dependencies.extend(self.process_node(node, definitions, &new_scope));

            let mut cursor = node.walk();
            let mut children: Vec<Node<'a>> = Vec::new();
            for child in node.children(&mut cursor) {
                children.push(child);
            }
            for child in children.into_iter().rev() {
                stack.push((child, new_scope.clone()));
            }
        }

        Ok(dependencies)
    }

    fn process_node(
        &self,
        node: Node<'a>,
        definitions: &[Definition],
        current_scope: &Option<String>,
    ) -> Vec<Dependency>;

    fn determine_scope(&self, node: &Node<'a>, parent_scope: &Option<String>) -> Option<String>;

    fn handle_identifier(
        &self,
        node: Node<'a>,
        dependencies: &mut Vec<Dependency>,
        definitions: &[Definition],
        current_scope: &Option<String>,
    );

    fn handle_call_expression(
        &self,
        node: Node<'a>,
        dependencies: &mut Vec<Dependency>,
        definitions: &[Definition],
        current_scope: &Option<String>,
    );

    fn handle_field_expression(
        &self,
        node: Node<'a>,
        dependencies: &mut Vec<Dependency>,
        definitions: &[Definition],
        current_scope: &Option<String>,
    );

    fn handle_struct_expression(
        &self,
        node: Node<'a>,
        dependencies: &mut Vec<Dependency>,
        definitions: &[Definition],
        current_scope: &Option<String>,
    );

    fn handle_metavariable(
        &self,
        node: Node<'a>,
        dependencies: &mut Vec<Dependency>,
        definitions: &[Definition],
        current_scope: &Option<String>,
    );

    #[allow(clippy::too_many_arguments)]
    fn add_dependency_if_needed(
        &self,
        dependencies: &mut Vec<Dependency>,
        node: Node<'a>,
        source_code: &'a str,
        definitions: &[Definition],
        current_scope: &Option<String>,
        dependency_type: DependencyType,
        context: Option<String>,
    ) {
        let source_line = node.start_position().row + 1;
        let symbol = node
            .utf8_text(source_code.as_bytes())
            .unwrap()
            .trim()
            .to_string();

        if let Some(def) = find_definition_in_scope(definitions, &symbol, current_scope) {
            let target_line = def.line_number;
            if source_line != target_line {
                dependencies.push(Dependency {
                    source_line,
                    target_line,
                    symbol,
                    dependency_type,
                    context,
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
