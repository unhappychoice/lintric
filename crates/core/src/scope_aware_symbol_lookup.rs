use crate::models::{
    scope::{ScopeId, ScopeTree},
    Definition,
};
use std::collections::HashMap;

#[derive(Debug)]
pub struct ScopeAwareSymbolLookup {
    scope_tree: ScopeTree,
}

impl ScopeAwareSymbolLookup {
    pub fn new(scope_tree: ScopeTree) -> Self {
        Self { scope_tree }
    }

    pub fn lookup_in_scope_chain(&self, scope_id: ScopeId, symbol: &str) -> Vec<Definition> {
        let mut definitions = Vec::new();
        let mut current_scope_id = scope_id;

        while let Some(scope) = self.scope_tree.get_scope(current_scope_id) {
            if let Some(scope_definitions) = scope.get_symbols(symbol) {
                for definition in scope_definitions {
                    definitions.push(definition.clone());
                }
            }

            if let Some(parent_id) = scope.parent {
                current_scope_id = parent_id;
            } else {
                break;
            }
        }

        definitions
    }

    pub fn get_shadowing_chain(
        &self,
        scope_id: ScopeId,
        symbol: &str,
    ) -> Vec<(ScopeId, Definition)> {
        let mut chain = Vec::new();
        let mut current_scope_id = scope_id;

        while let Some(scope) = self.scope_tree.get_scope(current_scope_id) {
            if let Some(definitions) = scope.get_symbols(symbol) {
                for definition in definitions {
                    chain.push((current_scope_id, definition.clone()));
                }
            }

            if let Some(parent_id) = scope.parent {
                current_scope_id = parent_id;
            } else {
                break;
            }
        }

        chain
    }

    pub fn is_shadowed(&self, definition: &Definition, at_scope: ScopeId) -> bool {
        let def_scope = self.find_definition_scope(definition);
        if def_scope.is_none() {
            return false;
        }

        let def_scope_id = def_scope.unwrap();

        if def_scope_id == at_scope {
            return false;
        }

        let chain = self.get_shadowing_chain(at_scope, &definition.name);

        for (scope_id, _) in chain {
            if scope_id != def_scope_id && self.is_nested_scope(scope_id, def_scope_id) {
                return true;
            }
        }

        false
    }

    pub fn get_visible_definitions_at_scope(
        &self,
        scope_id: ScopeId,
        symbol: &str,
    ) -> Vec<Definition> {
        let chain = self.get_shadowing_chain(scope_id, symbol);
        let mut visible_definitions = Vec::new();
        let mut seen_scopes = std::collections::HashSet::new();

        for (chain_scope_id, definition) in chain {
            if !seen_scopes.contains(&chain_scope_id) {
                seen_scopes.insert(chain_scope_id);
                visible_definitions.push(definition);

                if self.is_same_or_inner_scope(chain_scope_id, scope_id) {
                    break;
                }
            }
        }

        visible_definitions
    }

    pub fn calculate_scope_distance(
        &self,
        from_scope: ScopeId,
        to_scope: ScopeId,
    ) -> Option<usize> {
        if from_scope == to_scope {
            return Some(0);
        }

        let mut distance = 0;
        let mut current_scope = from_scope;

        while let Some(scope) = self.scope_tree.get_scope(current_scope) {
            if current_scope == to_scope {
                return Some(distance);
            }

            if let Some(parent_id) = scope.parent {
                current_scope = parent_id;
                distance += 1;
            } else {
                break;
            }
        }

        None
    }

    pub fn find_closest_common_scope(&self, scope1: ScopeId, scope2: ScopeId) -> Option<ScopeId> {
        let ancestors1 = self.get_scope_ancestors(scope1);
        let ancestors2 = self.get_scope_ancestors(scope2);

        for ancestor1 in &ancestors1 {
            if ancestors2.contains(ancestor1) {
                return Some(*ancestor1);
            }
        }

        None
    }

    pub fn get_scope_depth(&self, scope_id: ScopeId) -> usize {
        let mut depth = 0;
        let mut current_scope_id = scope_id;

        while let Some(scope) = self.scope_tree.get_scope(current_scope_id) {
            if let Some(parent_id) = scope.parent {
                depth += 1;
                current_scope_id = parent_id;
            } else {
                break;
            }
        }

        depth
    }

    pub fn get_symbols_with_scope_info(
        &self,
        scope_id: ScopeId,
    ) -> HashMap<String, Vec<(Definition, ScopeId, usize)>> {
        let mut symbols_info = HashMap::new();

        if let Some(scope) = self.scope_tree.get_scope(scope_id) {
            for (symbol_name, definitions) in &scope.symbols {
                let mut symbol_info = Vec::new();
                for definition in definitions {
                    let scope_depth = self.get_scope_depth(scope_id);
                    symbol_info.push((definition.clone(), scope_id, scope_depth));
                }
                symbols_info.insert(symbol_name.clone(), symbol_info);
            }
        }

        symbols_info
    }

    fn find_definition_scope(&self, definition: &Definition) -> Option<ScopeId> {
        self.scope_tree.find_scope_at_position(&definition.position)
    }

    fn is_nested_scope(&self, potential_inner: ScopeId, potential_outer: ScopeId) -> bool {
        let mut current_scope_id = potential_inner;

        while let Some(scope) = self.scope_tree.get_scope(current_scope_id) {
            if let Some(parent_id) = scope.parent {
                if parent_id == potential_outer {
                    return true;
                }
                current_scope_id = parent_id;
            } else {
                break;
            }
        }

        false
    }

    fn is_same_or_inner_scope(&self, scope1: ScopeId, scope2: ScopeId) -> bool {
        scope1 == scope2 || self.is_nested_scope(scope1, scope2)
    }

    fn get_scope_ancestors(&self, scope_id: ScopeId) -> Vec<ScopeId> {
        let mut ancestors = vec![scope_id];
        let mut current_scope_id = scope_id;

        while let Some(scope) = self.scope_tree.get_scope(current_scope_id) {
            if let Some(parent_id) = scope.parent {
                ancestors.push(parent_id);
                current_scope_id = parent_id;
            } else {
                break;
            }
        }

        ancestors
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{scope::ScopeType, DefinitionType, Position};

    fn create_test_position(line: usize, column: usize) -> Position {
        Position {
            start_line: line,
            start_column: column,
            end_line: line,
            end_column: column + 1,
        }
    }

    fn create_test_definition(name: &str, line: usize) -> Definition {
        Definition {
            name: name.to_string(),
            definition_type: DefinitionType::VariableDefinition,
            position: create_test_position(line, 1),
        }
    }

    #[test]
    fn test_scope_aware_lookup_basic() {
        let mut scope_tree = ScopeTree::new();

        let func_scope_id = scope_tree.create_scope(
            Some(0),
            ScopeType::Function,
            create_test_position(5, 1),
            create_test_position(15, 1),
        );

        let block_scope_id = scope_tree.create_scope(
            Some(func_scope_id),
            ScopeType::Block,
            create_test_position(8, 1),
            create_test_position(12, 1),
        );

        let global_def = create_test_definition("x", 1);
        let func_def = create_test_definition("x", 6);
        let block_def = create_test_definition("x", 9);

        if let Some(global_scope) = scope_tree.get_scope_mut(0) {
            global_scope.add_symbol("x".to_string(), global_def);
        }

        if let Some(func_scope) = scope_tree.get_scope_mut(func_scope_id) {
            func_scope.add_symbol("x".to_string(), func_def);
        }

        if let Some(block_scope) = scope_tree.get_scope_mut(block_scope_id) {
            block_scope.add_symbol("x".to_string(), block_def.clone());
        }

        let lookup = ScopeAwareSymbolLookup::new(scope_tree);

        let definitions = lookup.lookup_in_scope_chain(block_scope_id, "x");
        assert_eq!(definitions.len(), 3);

        let chain = lookup.get_shadowing_chain(block_scope_id, "x");
        assert_eq!(chain.len(), 3);
        assert_eq!(chain[0].1.position.start_line, 9);

        let distance = lookup.calculate_scope_distance(block_scope_id, 0);
        assert_eq!(distance, Some(2));

        let depth = lookup.get_scope_depth(block_scope_id);
        assert_eq!(depth, 2);
    }

    #[test]
    fn test_shadowing_detection() {
        let mut scope_tree = ScopeTree::new();

        let func_scope_id = scope_tree.create_scope(
            Some(0),
            ScopeType::Function,
            create_test_position(5, 1),
            create_test_position(15, 1),
        );

        let global_def = create_test_definition("variable", 1);
        let func_def = create_test_definition("variable", 6);

        if let Some(global_scope) = scope_tree.get_scope_mut(0) {
            global_scope.add_symbol("variable".to_string(), global_def.clone());
        }

        if let Some(func_scope) = scope_tree.get_scope_mut(func_scope_id) {
            func_scope.add_symbol("variable".to_string(), func_def);
        }

        let lookup = ScopeAwareSymbolLookup::new(scope_tree);

        let is_shadowed = lookup.is_shadowed(&global_def, func_scope_id);
        assert!(is_shadowed);
    }

    #[test]
    fn test_visible_definitions() {
        let mut scope_tree = ScopeTree::new();

        let func_scope_id = scope_tree.create_scope(
            Some(0),
            ScopeType::Function,
            create_test_position(5, 1),
            create_test_position(15, 1),
        );

        let global_def = create_test_definition("var", 1);
        let func_def = create_test_definition("var", 6);

        if let Some(global_scope) = scope_tree.get_scope_mut(0) {
            global_scope.add_symbol("var".to_string(), global_def);
        }

        if let Some(func_scope) = scope_tree.get_scope_mut(func_scope_id) {
            func_scope.add_symbol("var".to_string(), func_def.clone());
        }

        let lookup = ScopeAwareSymbolLookup::new(scope_tree);

        let visible = lookup.get_visible_definitions_at_scope(func_scope_id, "var");
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].position.start_line, 6);
    }

    #[test]
    fn test_common_scope_finding() {
        let mut scope_tree = ScopeTree::new();

        let func1_scope_id = scope_tree.create_scope(
            Some(0),
            ScopeType::Function,
            create_test_position(5, 1),
            create_test_position(10, 1),
        );

        let func2_scope_id = scope_tree.create_scope(
            Some(0),
            ScopeType::Function,
            create_test_position(15, 1),
            create_test_position(20, 1),
        );

        let lookup = ScopeAwareSymbolLookup::new(scope_tree);

        let common_scope = lookup.find_closest_common_scope(func1_scope_id, func2_scope_id);
        assert_eq!(common_scope, Some(0));
    }
}
