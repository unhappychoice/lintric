use std::path::Path;
use tree_sitter::Language as TreeSitterLanguage;

#[derive(Debug, Clone, PartialEq)]
pub enum Language {
    Rust,
    TypeScript,
    TSX,
}

impl Language {
    pub fn from_extension(path: &Path) -> Option<Self> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(|ext_str| match ext_str {
                "rs" => Some(Language::Rust),
                "ts" | "js" => Some(Language::TypeScript),
                "tsx" | "jsx" => Some(Language::TSX),
                _ => None,
            })
    }

    pub fn get_tree_sitter_language(&self) -> TreeSitterLanguage {
        match self {
            Language::Rust => tree_sitter_rust::LANGUAGE.into(),
            Language::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            Language::TSX => tree_sitter_typescript::LANGUAGE_TSX.into(),
        }
    }
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
