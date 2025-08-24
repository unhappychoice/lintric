use lintric_core::languages::rust::dependency_resolver::{
    ImportResolver, ModuleResolver, VisibilityChecker,
};
use lintric_core::models::{
    Definition, DefinitionType, ImportInfo, ImportType, ModuleTree, Position, Visibility,
};

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

// Integration tests (advanced module functionality)
mod integration_tests {
    use super::*;

    #[test]
    fn test_complex_visibility_rules() {
        let mut module_tree = ModuleTree::new();

        let parent_module = module_tree.add_module("parent".to_string(), Some(0), None);
        let child_module = module_tree.add_module("child".to_string(), Some(parent_module), None);

        let mut definition_visibility = std::collections::HashMap::new();
        definition_visibility.insert(
            format!("{}:pub_super_item", child_module),
            Visibility::PubSuper,
        );

        let visibility_checker = VisibilityChecker::new(module_tree, definition_visibility);

        let definition = Definition {
            name: "pub_super_item".to_string(),
            position: Position {
                start_line: 1,
                start_column: 1,
                end_line: 1,
                end_column: 14,
            },
            definition_type: DefinitionType::FunctionDefinition,
        };

        // pub(super) item should be accessible to parent module
        assert!(visibility_checker.is_accessible(&definition, child_module, parent_module));
        // but not to sibling modules or external modules
        assert!(!visibility_checker.is_accessible(&definition, child_module, 0));
    }

    #[test]
    fn test_cross_module_field_access() {
        let mut module_tree = ModuleTree::new();

        let structs_module = module_tree.add_module("structs".to_string(), Some(0), None);
        let main_module = module_tree.add_module("main".to_string(), Some(0), None);

        // Define a public struct in structs module
        let struct_def = Definition {
            name: "PublicStruct".to_string(),
            position: Position {
                start_line: 1,
                start_column: 1,
                end_line: 5,
                end_column: 1,
            },
            definition_type: DefinitionType::StructDefinition,
        };

        module_tree.add_export(structs_module, "PublicStruct".to_string(), struct_def);

        // Import it in main module
        let import = ImportInfo {
            imported_symbol: "PublicStruct".to_string(),
            source_module: "structs".to_string(),
            alias: None,
            import_type: ImportType::Named("PublicStruct".to_string()),
            visibility: Visibility::Private,
            position: Position {
                start_line: 1,
                start_column: 1,
                end_line: 1,
                end_column: 20,
            },
        };

        module_tree.add_import(main_module, import);

        // Verify cross-module access
        assert!(module_tree.modules[&structs_module]
            .exports
            .contains_key("PublicStruct"));
        assert_eq!(module_tree.modules[&main_module].imports.len(), 1);
    }

    #[test]
    fn test_module_re_exports() {
        let mut module_tree = ModuleTree::new();

        let core_module = module_tree.add_module("core".to_string(), Some(0), None);
        let lib_module = module_tree.add_module("lib".to_string(), Some(0), None);

        // Core module exports something
        let core_function = Definition {
            name: "core_function".to_string(),
            position: Position {
                start_line: 10,
                start_column: 1,
                end_line: 12,
                end_column: 1,
            },
            definition_type: DefinitionType::FunctionDefinition,
        };

        module_tree.add_export(
            core_module,
            "core_function".to_string(),
            core_function.clone(),
        );

        // Lib module imports and re-exports it
        let import = ImportInfo {
            imported_symbol: "core_function".to_string(),
            source_module: "core".to_string(),
            alias: None,
            import_type: ImportType::Named("core_function".to_string()),
            visibility: Visibility::Public,
            position: Position {
                start_line: 1,
                start_column: 1,
                end_line: 1,
                end_column: 25,
            },
        };

        module_tree.add_import(lib_module, import);
        module_tree.add_export(lib_module, "core_function".to_string(), core_function);

        // Both modules should now export the function
        assert!(module_tree.modules[&core_module]
            .exports
            .contains_key("core_function"));
        assert!(module_tree.modules[&lib_module]
            .exports
            .contains_key("core_function"));
    }

    #[test]
    fn test_wildcard_imports() {
        let mut module_tree = ModuleTree::new();

        let utils_module = module_tree.add_module("utils".to_string(), Some(0), None);
        let main_module = module_tree.add_module("main".to_string(), Some(0), None);

        // Utils module exports multiple items
        let helper_def = Definition {
            name: "helper".to_string(),
            position: Position {
                start_line: 1,
                start_column: 1,
                end_line: 3,
                end_column: 1,
            },
            definition_type: DefinitionType::FunctionDefinition,
        };

        let formatter_def = Definition {
            name: "formatter".to_string(),
            position: Position {
                start_line: 5,
                start_column: 1,
                end_line: 7,
                end_column: 1,
            },
            definition_type: DefinitionType::FunctionDefinition,
        };

        module_tree.add_export(utils_module, "helper".to_string(), helper_def);
        module_tree.add_export(utils_module, "formatter".to_string(), formatter_def);

        // Main module does wildcard import
        let wildcard_import = ImportInfo {
            imported_symbol: "*".to_string(),
            source_module: "utils".to_string(),
            alias: None,
            import_type: ImportType::Wildcard,
            visibility: Visibility::Private,
            position: Position {
                start_line: 1,
                start_column: 1,
                end_line: 1,
                end_column: 15,
            },
        };

        module_tree.add_import(main_module, wildcard_import);

        // Verify wildcard import was added
        assert_eq!(module_tree.modules[&main_module].imports.len(), 1);
        assert!(matches!(
            module_tree.modules[&main_module].imports[0].import_type,
            ImportType::Wildcard
        ));
    }

    #[test]
    fn test_use_statement_resolution() {
        let mut module_tree = ModuleTree::new();
        let resolver = ModuleResolver::new();

        // Create nested module structure: crate::database::models
        let database_module = module_tree.add_module("database".to_string(), Some(0), None);
        let models_module =
            module_tree.add_module("models".to_string(), Some(database_module), None);

        // Add some exports to models
        let user_model = Definition {
            name: "User".to_string(),
            position: Position {
                start_line: 1,
                start_column: 1,
                end_line: 10,
                end_column: 1,
            },
            definition_type: DefinitionType::StructDefinition,
        };

        module_tree.add_export(models_module, "User".to_string(), user_model);

        // Test path resolution
        assert_eq!(
            module_tree.find_module_by_path("database"),
            Some(database_module)
        );
        assert_eq!(
            module_tree.find_module_by_path("database::models"),
            Some(models_module)
        );
        assert!(module_tree.modules[&models_module]
            .exports
            .contains_key("User"));

        // Test resolver functionality
        let new_resolver_tree = resolver.get_module_tree();
        assert_eq!(new_resolver_tree.root_module, 0);
    }
}
