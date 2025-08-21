use crate::GenerationContext;
use lintric_core::models::language::Language;

/// Information about a language for test generation
#[derive(Debug, Clone)]
pub struct LanguageInfo {
    pub enum_variant: &'static str,
    pub folder_name: &'static str,
    pub display_name: &'static str,
}

/// Trait for language-specific test generation plugins
pub trait LanguagePlugin {
    /// Get the language this plugin handles
    fn language(&self) -> Language;

    /// Get language information for test generation
    fn language_info(&self) -> LanguageInfo;

    /// Generate a code snippet for the given node type
    fn generate_snippet(&self, node_type: &str, context: &mut GenerationContext) -> Option<String>;

    /// Generate validation code for a specific node type in s-expression
    fn generate_node_type_validation(&self, node_type: &str) -> String;

    /// Get the file paths for this language
    fn get_file_paths(
        &self,
        base_path: &std::path::Path,
    ) -> (std::path::PathBuf, std::path::PathBuf) {
        let lang_info = self.language_info();
        let input_path = base_path
            .parent()
            .unwrap()
            .join("../tmp/node_types")
            .join(lang_info.folder_name)
            .join("node-types.json");

        // Special handling for output file names to match existing convention
        let output_filename = match lang_info.folder_name {
            "ts" => "typescript.rs",
            folder => &format!("{}.rs", folder),
        };
        let output_path = base_path.join("tests").join(output_filename);
        (input_path, output_path)
    }
}

/// Registry for language plugins
pub struct LanguagePluginRegistry {
    plugins: Vec<Box<dyn LanguagePlugin>>,
}

impl LanguagePluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    pub fn register<P: LanguagePlugin + 'static>(&mut self, plugin: P) {
        self.plugins.push(Box::new(plugin));
    }

    pub fn get_plugin(&self, language: Language) -> Option<&dyn LanguagePlugin> {
        self.plugins
            .iter()
            .find(|p| p.language() == language)
            .map(|p| p.as_ref())
    }

    pub fn all_plugins(&self) -> &[Box<dyn LanguagePlugin>] {
        &self.plugins
    }
}

impl Default for LanguagePluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}
