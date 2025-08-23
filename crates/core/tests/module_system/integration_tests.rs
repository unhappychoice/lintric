use lintric_core::dependency_resolver::{ImportResolver, VisibilityChecker};
use lintric_core::models::{
    Definition, DefinitionType, ImportInfo, ImportType, ModuleTree, Position, Visibility,
};
use std::collections::HashMap;

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_complex_visibility_rules() {
        let mut module_tree = ModuleTree::new();

        // Create outer module
        let outer_module = module_tree.add_module("outer".to_string(), Some(0), None);

        // Create inner module within outer
        let inner_module = module_tree.add_module("inner".to_string(), Some(outer_module), None);

        // Add public function to inner module
        let public_inner_def = Definition {
            name: "public_inner".to_string(),
            position: Position {
                start_line: 1,
                start_column: 1,
                end_line: 1,
                end_column: 12,
            },
            definition_type: DefinitionType::FunctionDefinition,
        };
        module_tree.add_export(inner_module, "public_inner".to_string(), public_inner_def);

        // Add private function to inner module
        let private_inner_def = Definition {
            name: "private_inner".to_string(),
            position: Position {
                start_line: 2,
                start_column: 1,
                end_line: 2,
                end_column: 13,
            },
            definition_type: DefinitionType::FunctionDefinition,
        };
        module_tree.add_export(inner_module, "private_inner".to_string(), private_inner_def);

        // Add pub(crate) function to inner module
        let crate_visible_def = Definition {
            name: "crate_visible".to_string(),
            position: Position {
                start_line: 3,
                start_column: 1,
                end_line: 3,
                end_column: 13,
            },
            definition_type: DefinitionType::FunctionDefinition,
        };
        module_tree.add_export(inner_module, "crate_visible".to_string(), crate_visible_def);

        // Add pub(super) function to inner module
        let parent_visible_def = Definition {
            name: "parent_visible".to_string(),
            position: Position {
                start_line: 4,
                start_column: 1,
                end_line: 4,
                end_column: 14,
            },
            definition_type: DefinitionType::FunctionDefinition,
        };
        module_tree.add_export(
            inner_module,
            "parent_visible".to_string(),
            parent_visible_def,
        );

        let mut definition_visibility = HashMap::new();
        definition_visibility.insert("private_inner".to_string(), Visibility::Private);
        definition_visibility.insert("crate_visible".to_string(), Visibility::PubCrate);
        definition_visibility.insert("parent_visible".to_string(), Visibility::PubSuper);
        definition_visibility.insert("public_inner".to_string(), Visibility::Public);

        let visibility_checker = VisibilityChecker::new(module_tree.clone(), definition_visibility);

        // Test visibility from root module
        assert!(visibility_checker.check_cross_module_access(0, inner_module, &Visibility::Public));
        assert!(!visibility_checker.check_cross_module_access(
            0,
            inner_module,
            &Visibility::Private
        ));
        assert!(visibility_checker.check_cross_module_access(
            0,
            inner_module,
            &Visibility::PubCrate
        ));

        // Test visibility from outer module to inner
        assert!(visibility_checker.check_cross_module_access(
            outer_module,
            inner_module,
            &Visibility::Public
        ));
        assert!(!visibility_checker.check_cross_module_access(
            outer_module,
            inner_module,
            &Visibility::Private
        ));
        assert!(visibility_checker.check_cross_module_access(
            outer_module,
            inner_module,
            &Visibility::PubSuper
        ));
    }

    #[test]
    fn test_use_statement_resolution() {
        let mut module_tree = ModuleTree::new();

        // Create std collections module structure
        let std_module = module_tree.add_module("std".to_string(), Some(0), None);
        let collections_module =
            module_tree.add_module("collections".to_string(), Some(std_module), None);

        // Add HashMap to collections
        let hashmap_def = Definition {
            name: "HashMap".to_string(),
            position: Position {
                start_line: 1,
                start_column: 1,
                end_line: 1,
                end_column: 7,
            },
            definition_type: DefinitionType::StructDefinition,
        };
        module_tree.add_export(
            collections_module,
            "HashMap".to_string(),
            hashmap_def.clone(),
        );

        // Create utils module
        let utils_module = module_tree.add_module("utils".to_string(), Some(0), None);

        // Add helper function to utils
        let helper_def = Definition {
            name: "helper".to_string(),
            position: Position {
                start_line: 1,
                start_column: 1,
                end_line: 1,
                end_column: 6,
            },
            definition_type: DefinitionType::FunctionDefinition,
        };
        module_tree.add_export(utils_module, "helper".to_string(), helper_def.clone());

        // Create app module with imports
        let app_module = module_tree.add_module("app".to_string(), Some(0), None);

        // Add HashMap import
        let hashmap_import = ImportInfo {
            imported_symbol: "HashMap".to_string(),
            source_module: "std::collections".to_string(),
            alias: None,
            import_type: ImportType::Named("HashMap".to_string()),
            visibility: Visibility::Private,
            position: Position {
                start_line: 1,
                start_column: 1,
                end_line: 1,
                end_column: 25,
            },
        };
        module_tree.add_import(app_module, hashmap_import);

        // Add helper import
        let helper_import = ImportInfo {
            imported_symbol: "helper".to_string(),
            source_module: "utils".to_string(),
            alias: None,
            import_type: ImportType::Named("helper".to_string()),
            visibility: Visibility::Private,
            position: Position {
                start_line: 2,
                start_column: 1,
                end_line: 2,
                end_column: 15,
            },
        };
        module_tree.add_import(app_module, helper_import);

        let import_resolver = ImportResolver::new(module_tree.clone());

        // Test resolving HashMap from app module
        let resolved_hashmap = import_resolver.find_symbol_in_imports("HashMap", app_module);
        assert!(resolved_hashmap.is_some());
        assert_eq!(resolved_hashmap.unwrap().name, "HashMap");

        // Test resolving helper from app module
        let resolved_helper = import_resolver.find_symbol_in_imports("helper", app_module);
        assert!(resolved_helper.is_some());
        assert_eq!(resolved_helper.unwrap().name, "helper");

        // Test resolving non-imported symbol
        let non_existent = import_resolver.find_symbol_in_imports("NonExistent", app_module);
        assert!(non_existent.is_none());
    }

    #[test]
    fn test_cross_module_field_access() {
        let mut module_tree = ModuleTree::new();

        // Create auth module
        let auth_module = module_tree.add_module("auth".to_string(), Some(0), None);

        // Create User struct definition
        let user_def = Definition {
            name: "User".to_string(),
            position: Position {
                start_line: 1,
                start_column: 1,
                end_line: 1,
                end_column: 4,
            },
            definition_type: DefinitionType::StructDefinition,
        };
        module_tree.add_export(auth_module, "User".to_string(), user_def);

        // Add public name field
        let name_field_def = Definition {
            name: "name".to_string(),
            position: Position {
                start_line: 2,
                start_column: 5,
                end_line: 2,
                end_column: 9,
            },
            definition_type: DefinitionType::StructFieldDefinition,
        };
        module_tree.add_export(auth_module, "name".to_string(), name_field_def);

        // Add private email field
        let email_field_def = Definition {
            name: "email".to_string(),
            position: Position {
                start_line: 3,
                start_column: 5,
                end_line: 3,
                end_column: 10,
            },
            definition_type: DefinitionType::StructFieldDefinition,
        };
        module_tree.add_export(auth_module, "email".to_string(), email_field_def.clone());

        // Add public method
        let get_name_def = Definition {
            name: "get_name".to_string(),
            position: Position {
                start_line: 7,
                start_column: 12,
                end_line: 7,
                end_column: 20,
            },
            definition_type: DefinitionType::MethodDefinition,
        };
        module_tree.add_export(auth_module, "get_name".to_string(), get_name_def);

        // Add private method
        let get_email_def = Definition {
            name: "get_email".to_string(),
            position: Position {
                start_line: 11,
                start_column: 8,
                end_line: 11,
                end_column: 17,
            },
            definition_type: DefinitionType::MethodDefinition,
        };
        module_tree.add_export(auth_module, "get_email".to_string(), get_email_def.clone());

        // Create app module
        let app_module = module_tree.add_module("app".to_string(), Some(0), None);

        let mut definition_visibility = HashMap::new();
        definition_visibility.insert(format!("{}:User", auth_module), Visibility::Public);
        definition_visibility.insert(format!("{}:name", auth_module), Visibility::Public);
        definition_visibility.insert(format!("{}:email", auth_module), Visibility::Private);
        definition_visibility.insert(format!("{}:get_name", auth_module), Visibility::Public);
        definition_visibility.insert(format!("{}:get_email", auth_module), Visibility::Private);

        let visibility_checker = VisibilityChecker::new(module_tree.clone(), definition_visibility);

        // Test accessible symbols from app module to auth module
        let accessible_symbols = visibility_checker.get_accessible_symbols(app_module, auth_module);

        // Should include public items
        assert!(accessible_symbols.contains(&"User".to_string()));
        assert!(accessible_symbols.contains(&"name".to_string()));
        assert!(accessible_symbols.contains(&"get_name".to_string()));

        // Should not include private items
        assert!(!accessible_symbols.contains(&"email".to_string()));
        assert!(!accessible_symbols.contains(&"get_email".to_string()));

        // Test specific visibility checks
        assert!(!visibility_checker.is_accessible_between_modules(
            &email_field_def,
            app_module,
            auth_module
        ));
        assert!(!visibility_checker.is_accessible_between_modules(
            &get_email_def,
            app_module,
            auth_module
        ));
    }

    #[test]
    fn test_module_re_exports() {
        let mut module_tree = ModuleTree::new();

        // Create database module
        let db_module = module_tree.add_module("database".to_string(), Some(0), None);

        // Add Connection struct to database
        let connection_def = Definition {
            name: "Connection".to_string(),
            position: Position {
                start_line: 1,
                start_column: 1,
                end_line: 1,
                end_column: 10,
            },
            definition_type: DefinitionType::StructDefinition,
        };
        module_tree.add_export(db_module, "Connection".to_string(), connection_def.clone());

        // Create lib module (main module that re-exports)
        let lib_module = module_tree.add_module("lib".to_string(), Some(0), None);

        // Add re-export of Connection from database
        let reexport_import = ImportInfo {
            imported_symbol: "Connection".to_string(),
            source_module: "database".to_string(),
            alias: None,
            import_type: ImportType::Named("Connection".to_string()),
            visibility: Visibility::Public, // Re-exported as public
            position: Position {
                start_line: 1,
                start_column: 1,
                end_line: 1,
                end_column: 20,
            },
        };
        module_tree.add_import(lib_module, reexport_import);

        // Also add it as an export
        module_tree.add_export(lib_module, "Connection".to_string(), connection_def);

        let import_resolver = ImportResolver::new(module_tree.clone());

        // Test that Connection can be resolved from lib module
        let resolved_connection = import_resolver.find_symbol_in_imports("Connection", lib_module);
        assert!(resolved_connection.is_some());
        assert_eq!(resolved_connection.unwrap().name, "Connection");

        // Also should be available as export from lib
        assert!(module_tree.modules[&lib_module]
            .exports
            .contains_key("Connection"));
    }

    #[test]
    fn test_wildcard_imports() {
        let mut module_tree = ModuleTree::new();

        // Create prelude module
        let prelude_module = module_tree.add_module("prelude".to_string(), Some(0), None);

        // Add multiple items to prelude
        let vec_def = Definition {
            name: "Vec".to_string(),
            position: Position {
                start_line: 1,
                start_column: 1,
                end_line: 1,
                end_column: 3,
            },
            definition_type: DefinitionType::StructDefinition,
        };
        module_tree.add_export(prelude_module, "Vec".to_string(), vec_def);

        let option_def = Definition {
            name: "Option".to_string(),
            position: Position {
                start_line: 2,
                start_column: 1,
                end_line: 2,
                end_column: 6,
            },
            definition_type: DefinitionType::EnumDefinition,
        };
        module_tree.add_export(prelude_module, "Option".to_string(), option_def);

        // Create app module with wildcard import
        let app_module = module_tree.add_module("app".to_string(), Some(0), None);

        let wildcard_import = ImportInfo {
            imported_symbol: "*".to_string(),
            source_module: "prelude".to_string(),
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
        module_tree.add_import(app_module, wildcard_import);

        let import_resolver = ImportResolver::new(module_tree.clone());

        // Test use statement resolution for wildcard imports
        let prelude_definitions = import_resolver.resolve_use_statement("prelude", app_module);
        assert!(!prelude_definitions.is_empty());

        // Should contain both Vec and Option
        let names: Vec<String> = prelude_definitions.iter().map(|d| d.name.clone()).collect();
        assert!(names.contains(&"Vec".to_string()));
        assert!(names.contains(&"Option".to_string()));
    }
}
