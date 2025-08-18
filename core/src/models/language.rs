use std::path::Path;
use tree_sitter::Language as TreeSitterLanguage;
use tree_sitter_rust;
use tree_sitter_typescript;

#[derive(Debug, Clone, PartialEq)]
pub enum Language {
    Rust,
    TypeScript,
}

impl Language {
    pub fn from_extension(path: &Path) -> Option<Self> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(|ext_str| match ext_str {
                "rs" => Some(Language::Rust),
                "ts" | "tsx" | "js" | "jsx" => Some(Language::TypeScript),
                _ => None,
            })
    }

    pub fn get_tree_sitter_language(&self) -> TreeSitterLanguage {
        match self {
            Language::Rust => tree_sitter_rust::language(),
            Language::TypeScript => tree_sitter_typescript::language_typescript(),
        }
    }
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
