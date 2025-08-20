use lintric_generated_tests::{
    language_plugin::{LanguagePlugin, LanguagePluginRegistry},
    plugins::{RustPlugin, TsxPlugin, TypeScriptPlugin},
    test_helpers::generate_helper_functions,
    GenerationContext, NodeType,
};
use std::fs;

#[derive(Debug)]
enum GenerationError {
    IoError(std::io::Error),
    JsonError(serde_json::Error),
}

impl From<std::io::Error> for GenerationError {
    fn from(err: std::io::Error) -> Self {
        GenerationError::IoError(err)
    }
}

impl From<serde_json::Error> for GenerationError {
    fn from(err: serde_json::Error) -> Self {
        GenerationError::JsonError(err)
    }
}

impl std::fmt::Display for GenerationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GenerationError::IoError(e) => write!(f, "IO error: {}", e),
            GenerationError::JsonError(e) => write!(f, "JSON error: {}", e),
        }
    }
}

impl std::error::Error for GenerationError {}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let base_path = std::path::Path::new(&manifest_dir);

    let registry = create_plugin_registry();
    setup_directories(base_path).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
    process_all_plugins(&registry, base_path)
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

    println!("Test generation complete!");
    Ok(())
}

struct SnippetGenerator<'a> {
    plugin: &'a dyn LanguagePlugin,
}

impl<'a> SnippetGenerator<'a> {
    fn new(plugin: &'a dyn LanguagePlugin) -> Self {
        Self { plugin }
    }

    fn generate_snippet(&self, node_type: &str, context: &mut GenerationContext) -> Option<String> {
        if context.nesting_level > 5 || context.generated_types.contains(node_type) {
            return None;
        }

        context.nesting_level += 1;
        context.generated_types.insert(node_type.to_string());

        let result = self.plugin.generate_snippet(node_type, context);

        context.nesting_level -= 1;
        context.generated_types.remove(node_type);

        result
    }
}

struct TestGenerator<'a> {
    plugin: &'a dyn LanguagePlugin,
    snippet_generator: SnippetGenerator<'a>,
}

impl<'a> TestGenerator<'a> {
    fn new(plugin: &'a dyn LanguagePlugin) -> Self {
        let snippet_generator = SnippetGenerator::new(plugin);
        Self {
            plugin,
            snippet_generator,
        }
    }

    fn generate_tests(
        &self,
        node_types: &[NodeType],
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut context = GenerationContext::new();
        let test_cases: Vec<String> = node_types
            .iter()
            .filter_map(|node_type| self.generate_test_case(&node_type.node_type, &mut context))
            .collect();

        Ok(self.generate_test_module(&test_cases, &context))
    }

    fn generate_test_case(
        &self,
        node_type: &str,
        context: &mut GenerationContext,
    ) -> Option<String> {
        let snippet = self
            .snippet_generator
            .generate_snippet(node_type, context)?;

        // Create unique test name
        let base_name = node_type
            .replace('-', "_")
            .replace('"', "")
            .replace("'", "");
        let base_name = if base_name.starts_with('_') {
            base_name.trim_start_matches('_')
        } else {
            &base_name
        };
        let test_name = context.get_unique_name(&format!("test_generated_{}", base_name));

        // Get language info and validation
        let lang_info = self.plugin.language_info();
        let error_msg = format!("Failed to analyze code for node type: {}", node_type);
        let node_type_check = self.plugin.generate_node_type_validation(node_type);

        Some(format!(
            r#####"#[test]
fn {}() {{
    let source_code = r#"{}"#;

    assert_code_analysis_and_snapshot(
        source_code,
        {},
        "{}",
        "{}",
        "{}",
        |s_expr| {{{}
        }},
    );
}}"#####,
            test_name,
            snippet,
            lang_info.enum_variant,
            lang_info.folder_name,
            test_name,
            error_msg,
            node_type_check
        ))
    }

    fn generate_test_module(&self, test_cases: &[String], context: &GenerationContext) -> String {
        let lang_info = self.plugin.language_info();

        let mut excluded_list = context
            .excluded
            .iter()
            .map(|s| format!("// {}", s))
            .collect::<Vec<_>>();

        excluded_list.sort();

        let helper_functions = generate_helper_functions();

        format!(
            r#"// Generated tests for {} node types
// This file is auto-generated. Do not edit manually.

use lintric_core;

// Excluded node types (could not generate snippets):
{}

{}

{}
"#,
            lang_info.display_name,
            excluded_list.join("\n"),
            helper_functions,
            test_cases.join("\n\n")
        )
    }
}

fn load_node_types(file_path: &str) -> Result<Vec<NodeType>, GenerationError> {
    let content = fs::read_to_string(file_path)?;
    let node_types: Vec<NodeType> = serde_json::from_str(&content)?;
    Ok(node_types)
}

/// Initialize and configure the plugin registry
fn create_plugin_registry() -> LanguagePluginRegistry {
    let mut registry = LanguagePluginRegistry::new();
    registry.register(RustPlugin);
    registry.register(TypeScriptPlugin);
    registry.register(TsxPlugin);
    registry
}

/// Setup directory structure for test generation
fn setup_directories(base_path: &std::path::Path) -> Result<(), GenerationError> {
    fs::create_dir_all(base_path.join("tests"))?;
    Ok(())
}

/// Generate tests for a single language plugin
fn generate_tests_for_plugin(
    plugin: &dyn LanguagePlugin,
    base_path: &std::path::Path,
) -> Result<(), GenerationError> {
    let lang_info = plugin.language_info();
    println!("Generating tests for {}...", lang_info.display_name);

    let (input_path, output_path) = plugin.get_file_paths(base_path);
    let node_types = load_node_types(&input_path.to_string_lossy())?;

    let test_content = TestGenerator::new(plugin)
        .generate_tests(&node_types)
        .map_err(|e| GenerationError::IoError(std::io::Error::other(e.to_string())))?;

    fs::write(&output_path, test_content)?;
    println!("Generated: {}", output_path.to_string_lossy());
    Ok(())
}

/// Process all plugins in the registry
fn process_all_plugins(
    registry: &LanguagePluginRegistry,
    base_path: &std::path::Path,
) -> Result<(), GenerationError> {
    registry
        .all_plugins()
        .iter()
        .try_for_each(|plugin| generate_tests_for_plugin(plugin.as_ref(), base_path))
}
