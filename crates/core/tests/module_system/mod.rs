use lintric_core::languages::rust::dependency_resolver::{
    ImportResolver, ModuleResolver, VisibilityChecker,
};
use lintric_core::models::{
    Definition, DefinitionType, ImportInfo, ImportType, ModuleTree, Position, Visibility,
};

mod integration_tests;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_tree_creation() {
        let module_tree = ModuleTree::new();

        assert_eq!(module_tree.root_module, 0);
        assert!(module_tree.modules.contains_key(&0));
        assert_eq!(module_tree.modules[&0].name, "crate");
        assert_eq!(module_tree.module_paths.get("crate"), Some(&0));
    }

    #[test]
    fn test_module_tree_add_module() {
        let mut module_tree = ModuleTree::new();

        let auth_module =
            module_tree.add_module("auth".to_string(), Some(0), Some("auth.rs".to_string()));

        assert!(module_tree.modules.contains_key(&auth_module));
        assert_eq!(module_tree.modules[&auth_module].name, "auth");
        assert_eq!(module_tree.modules[&auth_module].parent, Some(0));
        assert_eq!(module_tree.modules[&0].children, vec![auth_module]);
        assert_eq!(module_tree.find_module_by_path("auth"), Some(auth_module));
    }

    #[test]
    fn test_nested_module_hierarchy() {
        let mut module_tree = ModuleTree::new();

        let outer_module = module_tree.add_module("outer".to_string(), Some(0), None);
        let inner_module = module_tree.add_module("inner".to_string(), Some(outer_module), None);

        assert_eq!(module_tree.find_module_by_path("outer"), Some(outer_module));
        assert_eq!(
            module_tree.find_module_by_path("outer::inner"),
            Some(inner_module)
        );

        assert_eq!(
            module_tree.modules[&inner_module].parent,
            Some(outer_module)
        );
        assert_eq!(
            module_tree.modules[&outer_module].children,
            vec![inner_module]
        );
    }

    #[test]
    fn test_import_info_creation() {
        let import = ImportInfo {
            imported_symbol: "HashMap".to_string(),
            source_module: "std::collections".to_string(),
            alias: None,
            import_type: ImportType::Named("HashMap".to_string()),
            visibility: Visibility::Private,
            position: Position {
                start_line: 1,
                start_column: 1,
                end_line: 1,
                end_column: 20,
            },
        };

        assert_eq!(import.imported_symbol, "HashMap");
        assert_eq!(import.source_module, "std::collections");
        assert!(matches!(import.import_type, ImportType::Named(_)));
        assert!(matches!(import.visibility, Visibility::Private));
    }

    #[test]
    fn test_visibility_types() {
        assert!(Visibility::Public.is_public());
        assert!(!Visibility::Private.is_public());
        assert!(Visibility::Private.is_private());
        assert!(Visibility::PubCrate.is_crate_visible());
        assert!(Visibility::PubSuper.is_super_visible());
    }

    #[test]
    fn test_module_resolver_creation() {
        let resolver = ModuleResolver::new();
        let module_tree = resolver.get_module_tree();

        assert_eq!(module_tree.root_module, 0);
        assert!(module_tree.modules.contains_key(&0));
    }

    #[test]
    fn test_import_resolver_basic() {
        let module_tree = ModuleTree::new();
        let import_resolver = ImportResolver::new(module_tree);

        // Basic test to ensure ImportResolver is properly constructed
        assert!(import_resolver
            .find_symbol_in_imports("nonexistent", 0)
            .is_none());
    }

    #[test]
    fn test_visibility_checker_public_access() {
        let module_tree = ModuleTree::new();
        let mut definition_visibility = std::collections::HashMap::new();
        definition_visibility.insert("0:test_function".to_string(), Visibility::Public);
        let visibility_checker = VisibilityChecker::new(module_tree, definition_visibility);

        let definition = Definition {
            name: "test_function".to_string(),
            position: Position {
                start_line: 1,
                start_column: 1,
                end_line: 1,
                end_column: 10,
            },
            definition_type: DefinitionType::FunctionDefinition,
        };

        assert!(visibility_checker.is_accessible(&definition, 0, 0));
    }

    #[test]
    fn test_visibility_checker_private_access() {
        let module_tree = ModuleTree::new();
        let mut definition_visibility = std::collections::HashMap::new();
        definition_visibility.insert("0:private_function".to_string(), Visibility::Private);
        let visibility_checker = VisibilityChecker::new(module_tree, definition_visibility);

        let definition = Definition {
            name: "private_function".to_string(),
            position: Position {
                start_line: 1,
                start_column: 1,
                end_line: 1,
                end_column: 10,
            },
            definition_type: DefinitionType::FunctionDefinition,
        };

        // Private items should not be accessible from different module
        assert!(!visibility_checker.is_accessible(&definition, 0, 1));
    }

    #[test]
    fn test_module_path_resolution() {
        let mut module_tree = ModuleTree::new();

        let auth_module = module_tree.add_module("auth".to_string(), Some(0), None);
        let user_module = module_tree.add_module("user".to_string(), Some(auth_module), None);

        assert_eq!(
            module_tree.get_module_path(auth_module),
            Some("auth".to_string())
        );
        assert_eq!(
            module_tree.get_module_path(user_module),
            Some("auth::user".to_string())
        );

        assert_eq!(module_tree.find_module_by_path("auth"), Some(auth_module));
        assert_eq!(
            module_tree.find_module_by_path("auth::user"),
            Some(user_module)
        );
    }

    #[test]
    fn test_module_exports() {
        let mut module_tree = ModuleTree::new();

        let auth_module = module_tree.add_module("auth".to_string(), Some(0), None);

        let definition = Definition {
            name: "User".to_string(),
            position: Position {
                start_line: 1,
                start_column: 1,
                end_line: 1,
                end_column: 4,
            },
            definition_type: DefinitionType::StructDefinition,
        };

        module_tree.add_export(auth_module, "User".to_string(), definition.clone());

        assert!(module_tree.modules[&auth_module]
            .exports
            .contains_key("User"));
        assert_eq!(
            module_tree.modules[&auth_module].exports["User"],
            definition
        );
    }

    #[test]
    fn test_module_imports() {
        let mut module_tree = ModuleTree::new();

        let app_module = module_tree.add_module("app".to_string(), Some(0), None);

        let import = ImportInfo {
            imported_symbol: "User".to_string(),
            source_module: "auth".to_string(),
            alias: None,
            import_type: ImportType::Named("User".to_string()),
            visibility: Visibility::Private,
            position: Position {
                start_line: 1,
                start_column: 1,
                end_line: 1,
                end_column: 10,
            },
        };

        module_tree.add_import(app_module, import.clone());

        assert_eq!(module_tree.modules[&app_module].imports.len(), 1);
        assert_eq!(module_tree.modules[&app_module].imports[0], import);
    }
}
