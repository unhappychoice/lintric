use std::fs;
use std::path::PathBuf;
use tree_sitter::{Parser as TreeSitterParser, Tree};

use crate::models::Language;
use crate::s_expression_formatter::SExpressionFormatter;

pub struct FileParser {
    file_content: String,
    language: Language,
}

impl FileParser {
    pub fn new(file_path: String) -> Result<Self, String> {
        let file_content = fs::read_to_string(&file_path)
            .map_err(|e| format!("Failed to read file {file_path}: {e}"))?;
        let language = Language::from_extension(&PathBuf::from(&file_path))
            .ok_or_else(|| format!("Unsupported file type for analysis: {file_path}"))?;
        Ok(FileParser {
            file_content,
            language,
        })
    }

    pub fn from_content(file_content: String, language: Language) -> Self {
        FileParser {
            file_content,
            language,
        }
    }

    pub fn parse(&self) -> Result<(String, Language, tree_sitter::Tree), String> {
        let tree = self.parse_file()?;
        Ok((self.file_content.clone(), self.language.clone(), tree))
    }

    pub fn parse_as_s_expression(&self) -> Result<String, String> {
        let tree = self.parse_file()?;
        let formatter = SExpressionFormatter::new(&self.file_content, self.language.clone());
        Ok(formatter.format_node(tree.root_node(), 0))
    }

    fn parse_file(&self) -> Result<Tree, String> {
        let mut parser = TreeSitterParser::new();

        let lang = &self.language.get_tree_sitter_language();

        parser
            .set_language(lang)
            .map_err(|e| format!("Error loading grammar: {e}"))?;

        let tree = parser
            .parse(&self.file_content, None)
            .ok_or_else(|| "Failed to parse the source code.".to_string())?;

        Ok(tree)
    }
}
